//! Scenario runner that executes headless diagnostics goals.

use crate::headless_assert::{
    evaluate_assert_exists, evaluate_assert_text_contains, AssertionResult, DiagnosticsSnapshot,
};
use crate::headless_report::HeadlessReport;
use crate::headless_runtime::{HeadlessRunConfig, HeadlessRuntime};
use crate::headless_scenario::{HeadlessScenario, ScenarioStep};
use anyhow::{bail, Result};

/// Temporal context passed into diagnostics probes.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProbeContext {
    pub elapsed_frames: u64,
    pub elapsed_ms: u64,
    pub step_index: usize,
}

/// Final outcome of a scenario run.
#[derive(Debug, Clone)]
pub enum RunOutcome {
    Passed { report: HeadlessReport },
    Failed { report: HeadlessReport },
}

impl RunOutcome {
    pub fn report(&self) -> &HeadlessReport {
        match self {
            RunOutcome::Passed { report } => report,
            RunOutcome::Failed { report } => report,
        }
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, RunOutcome::Failed { .. })
    }
}

/// Execute scenario JSON using a default empty snapshot probe.
pub fn run_scenario(input: &str) -> Result<RunOutcome> {
    let scenario = HeadlessScenario::from_json(input)?;
    if scenario.steps.iter().any(|s| {
        matches!(
            s,
            ScenarioStep::AssertExists { .. } | ScenarioStep::AssertTextContains { .. }
        )
    }) {
        bail!(
            "run_scenario cannot evaluate assertions without a probe; use run_scenario_with_probe"
        );
    }

    let mut probe = |_ctx: &ProbeContext| DiagnosticsSnapshot::default();
    run_loaded_scenario_with_probe(&scenario, HeadlessRunConfig::default(), &mut probe)
}

/// Execute scenario JSON with a custom snapshot probe.
pub fn run_scenario_with_probe<F>(
    input: &str,
    runtime_cfg: HeadlessRunConfig,
    mut probe: F,
) -> Result<RunOutcome>
where
    F: FnMut(&ProbeContext) -> DiagnosticsSnapshot,
{
    let scenario = HeadlessScenario::from_json(input)?;
    run_loaded_scenario_with_probe(&scenario, runtime_cfg, &mut probe)
}

/// Execute a pre-loaded scenario with a custom snapshot probe.
pub fn run_loaded_scenario_with_probe<F>(
    scenario: &HeadlessScenario,
    runtime_cfg: HeadlessRunConfig,
    probe: &mut F,
) -> Result<RunOutcome>
where
    F: FnMut(&ProbeContext) -> DiagnosticsSnapshot,
{
    let mut elapsed_frames: u64 = 0;
    let mut elapsed_ms: u64 = 0;
    let mut latest_snapshot: Option<DiagnosticsSnapshot> = None;
    let probe_every = runtime_cfg.probe_every_frames.max(1);

    for (step_index, step) in scenario.steps.iter().enumerate() {
        match step {
            ScenarioStep::Wait { ms } => {
                let frames = wait_frames(*ms, runtime_cfg.tick_ms);
                if frames > 0 {
                    let mut cfg = runtime_cfg;
                    cfg.max_frames = frames;
                    let mut sampled_frames = 0u32;
                    let mut remaining_ms = *ms;
                    HeadlessRuntime::run(cfg, |_| {
                        elapsed_frames = elapsed_frames.saturating_add(1);
                        let step_ms = remaining_ms.min(runtime_cfg.tick_ms);
                        elapsed_ms = elapsed_ms.saturating_add(step_ms);
                        remaining_ms = remaining_ms.saturating_sub(step_ms);
                        sampled_frames = sampled_frames.saturating_add(1);

                        if sampled_frames % probe_every == 0 || sampled_frames == frames {
                            latest_snapshot = Some(probe(&ProbeContext {
                                elapsed_frames,
                                elapsed_ms,
                                step_index,
                            }));
                        }
                    })?;
                } else {
                    latest_snapshot = Some(probe(&ProbeContext {
                        elapsed_frames,
                        elapsed_ms,
                        step_index,
                    }));
                }
            }
            ScenarioStep::Tick { frames } => {
                if *frames > 0 {
                    let mut cfg = runtime_cfg;
                    cfg.max_frames = *frames;
                    let mut sampled_frames = 0u32;
                    HeadlessRuntime::run(cfg, |_| {
                        elapsed_frames = elapsed_frames.saturating_add(1);
                        elapsed_ms = elapsed_ms.saturating_add(runtime_cfg.tick_ms);
                        sampled_frames = sampled_frames.saturating_add(1);

                        if sampled_frames % probe_every == 0 || sampled_frames == *frames {
                            latest_snapshot = Some(probe(&ProbeContext {
                                elapsed_frames,
                                elapsed_ms,
                                step_index,
                            }));
                        }
                    })?;
                } else {
                    latest_snapshot = Some(probe(&ProbeContext {
                        elapsed_frames,
                        elapsed_ms,
                        step_index,
                    }));
                }
            }
            ScenarioStep::AssertExists { id } => {
                let snapshot = ensure_snapshot(
                    &mut latest_snapshot,
                    probe,
                    ProbeContext {
                        elapsed_frames,
                        elapsed_ms,
                        step_index,
                    },
                );
                if let AssertionResult::Failed { message, .. } =
                    evaluate_assert_exists(id, snapshot)
                {
                    let report = HeadlessReport::failed(
                        "assert_exists",
                        step_index,
                        message,
                        elapsed_frames,
                        elapsed_ms,
                    );
                    return Ok(RunOutcome::Failed { report });
                }
            }
            ScenarioStep::AssertTextContains { id, value } => {
                let snapshot = ensure_snapshot(
                    &mut latest_snapshot,
                    probe,
                    ProbeContext {
                        elapsed_frames,
                        elapsed_ms,
                        step_index,
                    },
                );
                if let AssertionResult::Failed { message, .. } =
                    evaluate_assert_text_contains(id, value, snapshot)
                {
                    let report = HeadlessReport::failed(
                        "assert_text_contains",
                        step_index,
                        message,
                        elapsed_frames,
                        elapsed_ms,
                    );
                    return Ok(RunOutcome::Failed { report });
                }
            }
        }
    }

    Ok(RunOutcome::Passed {
        report: HeadlessReport::passed(elapsed_frames, elapsed_ms),
    })
}

fn ensure_snapshot<'a, F>(
    latest_snapshot: &'a mut Option<DiagnosticsSnapshot>,
    probe: &mut F,
    probe_ctx: ProbeContext,
) -> &'a DiagnosticsSnapshot
where
    F: FnMut(&ProbeContext) -> DiagnosticsSnapshot,
{
    latest_snapshot.get_or_insert_with(|| probe(&probe_ctx))
}

fn wait_frames(wait_ms: u64, tick_ms: u64) -> u32 {
    if wait_ms == 0 {
        return 0;
    }
    let tick = tick_ms.max(1);
    let frames = wait_ms.saturating_add(tick.saturating_sub(1)) / tick;
    frames.min(u32::MAX as u64) as u32
}
