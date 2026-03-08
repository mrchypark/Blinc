//! State-machine playbooks layered on top of the existing FSM runtime.

use anyhow::{bail, Context, Result};
use blinc_core::fsm::EventId;
use blinc_core::{FsmId, FsmRuntime, StateId, Transition};
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use crate::automation_session::{run_desktop_harness_scenario, run_headless_scenario};
use crate::headless_runtime::HeadlessRunConfig;
use crate::headless_scenario::{HeadlessScenario, ScenarioStep};
use crate::windowed::WindowedContext;
use crate::AutomationRun;

#[derive(Debug, Clone, Deserialize)]
pub struct Playbook {
    pub initial_state: String,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub execution: Vec<String>,
    #[serde(default)]
    pub transitions: Vec<PlaybookTransition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaybookTransition {
    #[serde(default)]
    pub name: Option<String>,
    pub from: String,
    pub event: String,
    pub to: String,
    #[serde(default)]
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone)]
pub struct CompiledPlaybook {
    pub initial_state: StateId,
    pub state_ids: HashMap<String, StateId>,
    pub event_ids: HashMap<String, EventId>,
    pub execution: Vec<String>,
    pub transitions: Vec<CompiledTransition>,
}

#[derive(Debug, Clone)]
pub struct CompiledTransition {
    pub name: String,
    pub from_state: StateId,
    pub event: EventId,
    pub to_state: StateId,
    pub steps: Vec<ScenarioStep>,
}

impl Playbook {
    pub fn from_yaml(input: &str) -> Result<Self> {
        let playbook: Self = serde_yaml::from_str(input)?;
        playbook.validate()?;
        Ok(playbook)
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read playbook {}", path.display()))?;
        Self::from_yaml(&raw)
            .with_context(|| format!("failed to parse playbook {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        let initial_state = self.initial_state.trim();
        if initial_state.is_empty() {
            bail!("playbook initial_state cannot be empty");
        }

        for state in &self.states {
            if state.trim().is_empty() {
                bail!("playbook states cannot contain empty names");
            }
        }

        for selector in &self.execution {
            if selector.trim().is_empty() {
                bail!("playbook execution steps cannot be empty");
            }
        }

        for transition in &self.transitions {
            if transition
                .name
                .as_deref()
                .is_some_and(|name| name.trim().is_empty())
            {
                bail!(
                    "transition {} cannot have an empty name",
                    transition_name(transition)
                );
            }
            if transition.from.trim().is_empty() {
                bail!(
                    "transition {} cannot have an empty from state",
                    transition_name(transition)
                );
            }
            if transition.event.trim().is_empty() {
                bail!(
                    "transition {} cannot have an empty event",
                    transition_name(transition)
                );
            }
            if transition.to.trim().is_empty() {
                bail!(
                    "transition {} cannot have an empty to state",
                    transition_name(transition)
                );
            }
        }

        Ok(())
    }

    pub fn compile(&self) -> Result<CompiledPlaybook> {
        self.validate()?;

        let initial_state = self.initial_state.trim();
        let explicit_states = if self.states.is_empty() {
            None
        } else {
            let mut known = BTreeSet::new();
            known.insert(initial_state.to_string());
            for state in &self.states {
                known.insert(state.clone());
            }
            Some(known)
        };

        let mut state_ids = HashMap::new();
        let mut event_ids = HashMap::new();
        let mut next_state_id: StateId = 0;
        let mut next_event_id: EventId = 0;

        let register_state = |name: &str,
                              state_ids: &mut HashMap<String, StateId>,
                              next_state_id: &mut StateId|
         -> StateId {
            if let Some(existing) = state_ids.get(name) {
                *existing
            } else {
                let assigned = *next_state_id;
                *next_state_id += 1;
                state_ids.insert(name.to_string(), assigned);
                assigned
            }
        };
        let register_event = |name: &str,
                              event_ids: &mut HashMap<String, EventId>,
                              next_event_id: &mut EventId|
         -> EventId {
            if let Some(existing) = event_ids.get(name) {
                *existing
            } else {
                let assigned = *next_event_id;
                *next_event_id += 1;
                event_ids.insert(name.to_string(), assigned);
                assigned
            }
        };

        let initial_state_id = register_state(initial_state, &mut state_ids, &mut next_state_id);

        if let Some(known_states) = explicit_states.as_ref() {
            for transition in &self.transitions {
                if !known_states.contains(&transition.from) {
                    bail!(
                        "transition {} references unknown state {}",
                        transition_name(transition),
                        transition.from
                    );
                }
                if !known_states.contains(&transition.to) {
                    bail!(
                        "transition {} references unknown state {}",
                        transition_name(transition),
                        transition.to
                    );
                }
            }
        }

        let mut compiled = Vec::with_capacity(self.transitions.len());
        for transition in &self.transitions {
            let from_state = register_state(&transition.from, &mut state_ids, &mut next_state_id);
            let to_state = register_state(&transition.to, &mut state_ids, &mut next_state_id);
            let event = register_event(&transition.event, &mut event_ids, &mut next_event_id);
            compiled.push(CompiledTransition {
                name: transition_name(transition),
                from_state,
                event,
                to_state,
                steps: transition.steps.clone(),
            });
        }

        Ok(CompiledPlaybook {
            initial_state: initial_state_id,
            state_ids,
            event_ids,
            execution: self.execution.clone(),
            transitions: compiled,
        })
    }

    pub fn resolve_embedded_paths(&mut self, base: &Path) {
        for transition in &mut self.transitions {
            for step in &mut transition.steps {
                step.resolve_artifact_paths(base);
            }
        }
    }

    pub fn export_mermaid(&self) -> Result<String> {
        let compiled = self.compile()?;
        let mut out = String::from("stateDiagram-v2\n");
        for transition in &compiled.transitions {
            out.push_str(&format!(
                "    {} --> {}: {}\n",
                state_name(&compiled.state_ids, transition.from_state),
                state_name(&compiled.state_ids, transition.to_state),
                event_name(&compiled.event_ids, transition.event)
            ));
        }
        Ok(out)
    }
}

impl CompiledPlaybook {
    pub fn instantiate_runtime(&self) -> (FsmRuntime, FsmId) {
        let transitions = self
            .transitions
            .iter()
            .map(|transition| {
                Transition::new(transition.from_state, transition.event, transition.to_state)
            })
            .collect();
        let mut runtime = FsmRuntime::new();
        let id = runtime.create_simple(self.initial_state, transitions);
        (runtime, id)
    }

    pub fn validate_execution_order(&self) -> Result<()> {
        let _ = self.execution_sequence()?;
        Ok(())
    }

    fn execution_sequence(&self) -> Result<Vec<&CompiledTransition>> {
        if !self.execution.is_empty() {
            return self.execution_sequence_from_path();
        }

        let (mut runtime, machine) = self.instantiate_runtime();
        let mut current_state = self.initial_state;
        let mut consumed = vec![false; self.transitions.len()];
        let mut ordered = Vec::with_capacity(self.transitions.len());

        loop {
            let candidate_indices = self
                .transitions
                .iter()
                .enumerate()
                .filter_map(|(index, transition)| {
                    (!consumed[index] && transition.from_state == current_state).then_some(index)
                })
                .collect::<Vec<_>>();

            match candidate_indices.as_slice() {
                [] => break,
                [index] => {
                    let transition = &self.transitions[*index];
                    let resulting_state = runtime
                        .send(machine, transition.event)
                        .with_context(|| format!("failed to send event {}", transition.event))?;
                    if resulting_state != transition.to_state {
                        bail!(
                            "transition {} expected state {} but got {}",
                            transition.name,
                            transition.to_state,
                            resulting_state
                        );
                    }
                    current_state = resulting_state;
                    consumed[*index] = true;
                    ordered.push(transition);
                }
                _ => {
                    bail!(
                        "playbook execution is ambiguous from state {}",
                        state_name(&self.state_ids, current_state)
                    );
                }
            }
        }

        if consumed.iter().any(|used| !*used) {
            bail!("playbook contains disconnected or branching transitions that require an explicit execution path");
        }

        Ok(ordered)
    }

    fn execution_sequence_from_path(&self) -> Result<Vec<&CompiledTransition>> {
        let (mut runtime, machine) = self.instantiate_runtime();
        let mut current_state = self.initial_state;
        let mut ordered = Vec::with_capacity(self.execution.len());

        for selector in &self.execution {
            let matches = self
                .transitions
                .iter()
                .filter(|transition| {
                    transition.from_state == current_state
                        && (transition.name == *selector
                            || event_name(&self.event_ids, transition.event) == *selector)
                })
                .collect::<Vec<_>>();

            let transition = match matches.as_slice() {
                [transition] => *transition,
                [] => {
                    bail!(
                        "execution path step {:?} does not match any transition from state {}",
                        selector,
                        state_name(&self.state_ids, current_state)
                    );
                }
                _ => {
                    bail!(
                        "execution path step {:?} matches multiple transitions from state {}",
                        selector,
                        state_name(&self.state_ids, current_state)
                    );
                }
            };

            let resulting_state = runtime
                .send(machine, transition.event)
                .with_context(|| format!("failed to send event {}", transition.event))?;
            if resulting_state != transition.to_state {
                bail!(
                    "transition {} expected state {} but got {}",
                    transition.name,
                    transition.to_state,
                    resulting_state
                );
            }

            current_state = resulting_state;
            ordered.push(transition);
        }

        Ok(ordered)
    }

    pub fn execution_scenario(&self) -> Result<HeadlessScenario> {
        let steps = self
            .execution_sequence()?
            .into_iter()
            .flat_map(|transition| transition.steps.clone())
            .collect();
        let scenario = HeadlessScenario { steps };
        scenario.validate()?;
        Ok(scenario)
    }

    pub fn flatten_scenario(&self) -> Result<HeadlessScenario> {
        self.execution_scenario()
    }
}

pub fn run_headless_playbook<F, E>(
    runtime_cfg: HeadlessRunConfig,
    playbook: &Playbook,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    run_playbook_with_mode(runtime_cfg, playbook, ui_builder, run_headless_scenario)
}

pub fn run_desktop_harness_playbook<F, E>(
    runtime_cfg: HeadlessRunConfig,
    playbook: &Playbook,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    run_playbook_with_mode(
        runtime_cfg,
        playbook,
        ui_builder,
        run_desktop_harness_scenario,
    )
}

fn run_playbook_with_mode<F, E, R>(
    runtime_cfg: HeadlessRunConfig,
    playbook: &Playbook,
    ui_builder: F,
    runner: R,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
    R: FnOnce(HeadlessRunConfig, &HeadlessScenario, F) -> Result<AutomationRun>,
{
    let compiled = playbook.compile()?;
    compiled.validate_execution_order()?;
    let scenario = compiled.execution_scenario()?;
    runner(runtime_cfg, &scenario, ui_builder)
}

fn transition_name(transition: &PlaybookTransition) -> String {
    transition
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}:{}", transition.from, transition.event))
}

fn state_name(state_ids: &HashMap<String, StateId>, state_id: StateId) -> String {
    state_ids
        .iter()
        .find_map(|(name, id): (&String, &StateId)| (*id == state_id).then(|| name.clone()))
        .unwrap_or_else(|| format!("state_{state_id}"))
}

fn event_name(event_ids: &HashMap<String, EventId>, event_id: EventId) -> String {
    event_ids
        .iter()
        .find_map(|(name, id): (&String, &EventId)| (*id == event_id).then(|| name.clone()))
        .unwrap_or_else(|| format!("event_{event_id}"))
}

#[cfg(test)]
mod tests {
    use super::Playbook;

    #[test]
    fn from_yaml_rejects_empty_initial_state() {
        let err = Playbook::from_yaml(
            r#"
initial_state: "   "
transitions: []
"#,
        )
        .expect_err("empty initial state should fail parsing validation");

        assert!(
            err.to_string().contains("initial_state cannot be empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_yaml_rejects_blank_transition_names() {
        let err = Playbook::from_yaml(
            r#"
initial_state: idle
transitions:
  - name: "   "
    from: idle
    event: submit
    to: done
"#,
        )
        .expect_err("blank transition names should fail validation");

        assert!(
            err.to_string().contains("cannot have an empty name"),
            "unexpected error: {err}"
        );
    }
}
