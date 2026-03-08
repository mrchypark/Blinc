use anyhow::Result;
use blinc_layout::selector::SemanticLocator;
use blinc_platform::AccessibilityRole;

use crate::frame_utils::wait_frames_for_duration;
use crate::headless_report::HeadlessReport;
use crate::headless_runtime::HeadlessRunConfig;
use crate::headless_scenario::{HeadlessScenario, ScenarioStep, ScenarioTarget};
use crate::windowed::WindowedContext;

use super::{
    AutomationFailure, AutomationLocator, AutomationRun, AutomationRuntimeMode, AutomationSession,
};

pub fn run_headless_scenario<F, E>(
    runtime_cfg: HeadlessRunConfig,
    scenario: &HeadlessScenario,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    run_scenario_with_mode(
        AutomationRuntimeMode::Headless,
        runtime_cfg,
        scenario,
        ui_builder,
    )
}

pub fn run_desktop_harness_scenario<F, E>(
    runtime_cfg: HeadlessRunConfig,
    scenario: &HeadlessScenario,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    run_scenario_with_mode(
        AutomationRuntimeMode::DesktopHarness,
        runtime_cfg,
        scenario,
        ui_builder,
    )
}

fn run_scenario_with_mode<F, E>(
    runtime_mode: AutomationRuntimeMode,
    runtime_cfg: HeadlessRunConfig,
    scenario: &HeadlessScenario,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    let mut session = match runtime_mode {
        AutomationRuntimeMode::Headless => AutomationSession::new_headless(runtime_cfg, ui_builder),
        AutomationRuntimeMode::DesktopHarness => {
            AutomationSession::new_desktop_harness(runtime_cfg, ui_builder)
        }
    };
    let mut elapsed_frames: u64 = 0;
    let mut elapsed_ms: u64 = 0;

    for (step_index, step) in scenario.steps.iter().enumerate() {
        let result = match step {
            ScenarioStep::Wait { ms } => {
                let frames = wait_frames_for_duration(*ms, runtime_cfg.tick_ms);
                session.tick_frames(frames)?;
                elapsed_frames += frames as u64;
                elapsed_ms += ms;
                Ok(())
            }
            ScenarioStep::Tick { frames } => {
                session.tick_frames(*frames)?;
                elapsed_frames += *frames as u64;
                elapsed_ms += runtime_cfg.tick_ms.saturating_mul(*frames as u64);
                Ok(())
            }
            ScenarioStep::Click { target, x, y } => match (x, y, target.is_empty()) {
                (Some(x), Some(y), true) => session.click_at(*x, *y),
                (None, None, _) => session.click(automation_locator_from_target(target)?),
                _ => Err(AutomationFailure {
                    code: "invalid_locator".to_string(),
                    message: "coordinate click steps require both x and y without locator fields"
                        .to_string(),
                    target: None,
                    trace_sequence: None,
                }),
            },
            ScenarioStep::Fill { target, value } => {
                session.fill(automation_locator_from_target(target)?, value)
            }
            ScenarioStep::Press { key } => session.press(key),
            ScenarioStep::Scroll { target, dx, dy } => {
                let locator = if target.id.is_some() || target.has_semantic_fields() {
                    Some(automation_locator_from_target(target)?)
                } else {
                    None
                };
                session.scroll(locator, *dx, *dy)
            }
            ScenarioStep::Snapshot { path } => {
                if let Some(path) = path.as_deref() {
                    session.write_snapshot_to_path(std::path::Path::new(path))?;
                }
                Ok(())
            }
            ScenarioStep::ExportTrace { path } => {
                if let Some(path) = path.as_deref() {
                    session.write_trace_to_path(std::path::Path::new(path))?;
                }
                Ok(())
            }
            ScenarioStep::AssertExists { target } => {
                session.assert_exists(automation_locator_from_target(target)?)
            }
            ScenarioStep::AssertTextContains { target, value } => {
                session.assert_text_contains(automation_locator_from_target(target)?, value)
            }
        };

        if let Err(failure) = result {
            session.stop_recording();
            return Ok(AutomationRun {
                report: HeadlessReport::failed(
                    &failure.code,
                    step_index,
                    failure.message,
                    elapsed_frames,
                    elapsed_ms,
                ),
                export: session.export_recording(),
            });
        }
    }

    session.stop_recording();
    Ok(AutomationRun {
        report: HeadlessReport::passed(elapsed_frames, elapsed_ms),
        export: session.export_recording(),
    })
}

fn parse_accessibility_role(input: &str) -> Option<AccessibilityRole> {
    match input.trim().to_ascii_lowercase().as_str() {
        "window" => Some(AccessibilityRole::Window),
        "group" => Some(AccessibilityRole::Group),
        "label" => Some(AccessibilityRole::Label),
        "button" => Some(AccessibilityRole::Button),
        "checkbox" => Some(AccessibilityRole::Checkbox),
        "text_input" | "textinput" | "textbox" => Some(AccessibilityRole::TextInput),
        "text_area" | "textarea" => Some(AccessibilityRole::TextArea),
        "image" => Some(AccessibilityRole::Image),
        _ => None,
    }
}

fn automation_locator_from_target(
    target: &ScenarioTarget,
) -> Result<AutomationLocator, AutomationFailure> {
    if let Some(id) = target.id.as_ref() {
        return Ok(AutomationLocator::id(id.clone()));
    }

    if !target.has_semantic_fields() {
        return Err(AutomationFailure {
            code: "invalid_locator".to_string(),
            message: "scenario step requires id or semantic locator fields".to_string(),
            target: None,
            trace_sequence: None,
        });
    }

    let semantic = &target.semantic;
    let mut locator = if let Some(role) = semantic.role.as_deref() {
        let Some(role) = parse_accessibility_role(role) else {
            return Err(AutomationFailure {
                code: "invalid_locator".to_string(),
                message: format!("unsupported accessibility role {role:?}"),
                target: None,
                trace_sequence: None,
            });
        };
        SemanticLocator::role(role)
    } else {
        SemanticLocator::default()
    };

    if let Some(text) = semantic.text.as_deref() {
        locator = locator.with_text(text);
    }
    if let Some(label) = semantic.label.as_deref() {
        locator = locator.with_label(label);
    }
    if let Some(placeholder) = semantic.placeholder.as_deref() {
        locator = locator.with_placeholder(placeholder);
    }
    if let Some(tag) = semantic.tag.as_deref() {
        locator = locator.with_tag(tag);
    }
    if let Some(within) = semantic.within.as_deref() {
        locator = locator.within(within);
    }
    if let Some(nth) = semantic.nth {
        locator = locator.nth(nth);
    }

    Ok(AutomationLocator::semantic(locator))
}
