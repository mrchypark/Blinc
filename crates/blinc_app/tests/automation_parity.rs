use blinc_app::prelude::*;
use blinc_app::{
    run_desktop_harness_scenario, run_headless_playbook, run_headless_scenario, HeadlessScenario,
    Playbook, ReportStatus,
};
use std::sync::{Mutex, Once};

fn automation_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().expect("automation parity lock")
}

fn ensure_theme() {
    static INIT: Once = Once::new();
    INIT.call_once(blinc_theme::ThemeState::init_default);
}

fn login_ui(ctx: &mut blinc_app::windowed::WindowedContext) -> impl ElementBuilder {
    let email = ctx.use_state_keyed("email", || text_input_state_with_placeholder("Email"));
    let email_data = email.get();
    let status = ctx.use_state_keyed("status", || "Signed out".to_string());

    div()
        .w(ctx.width)
        .h(ctx.height)
        .flex_col()
        .gap(16.0)
        .p(24.0)
        .child(
            text_input(&email_data)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
        .child(
            div()
                .id("login.submit")
                .on_click({
                    let email = email.clone();
                    let status = status.clone();
                    move |_| {
                        let value = email
                            .get()
                            .lock()
                            .expect("email input should lock")
                            .value
                            .clone();
                        if value.contains('@') {
                            status.set(format!("Signed in: {value}"));
                        } else {
                            status.set("Invalid email".to_string());
                        }
                    }
                })
                .child(text("Submit")),
        )
        .child(div().id("login.status").child(text(status.get())))
}

#[test]
fn login_scenario_matches_between_headless_and_desktop() {
    let _guard = automation_guard();
    ensure_theme();
    blinc_layout::widgets::blur_all_text_inputs();

    let scenario = HeadlessScenario::from_path(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/login_scenario.json"
    )))
    .expect("login scenario should load");

    let headless = run_headless_scenario(HeadlessRunConfig::default(), &scenario, login_ui)
        .expect("headless scenario should run");
    let desktop = run_desktop_harness_scenario(HeadlessRunConfig::default(), &scenario, login_ui)
        .expect("desktop harness scenario should run");

    assert!(matches!(headless.report.status, ReportStatus::Passed));
    assert!(matches!(desktop.report.status, ReportStatus::Passed));
    assert_eq!(
        headless.report.failed_step_index,
        desktop.report.failed_step_index
    );
    assert_eq!(headless.report.assertion, desktop.report.assertion);
}

#[test]
fn playbook_execution_emits_same_required_trace_fields_as_scenario_execution() {
    let _guard = automation_guard();
    ensure_theme();
    blinc_layout::widgets::blur_all_text_inputs();

    let scenario = HeadlessScenario::from_path(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/login_scenario.json"
    )))
    .expect("login scenario should load");
    let playbook = Playbook::from_path(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/login_playbook.yaml"
    )))
    .expect("login playbook should load");

    let scenario_run = run_headless_scenario(HeadlessRunConfig::default(), &scenario, login_ui)
        .expect("scenario execution should succeed");
    let playbook_run = run_headless_playbook(HeadlessRunConfig::default(), &playbook, login_ui)
        .expect("playbook execution should succeed");

    assert!(matches!(scenario_run.report.status, ReportStatus::Passed));
    assert!(matches!(playbook_run.report.status, ReportStatus::Passed));
    assert!(!scenario_run.export.snapshots.is_empty());
    assert!(!playbook_run.export.snapshots.is_empty());

    let scenario_commands = scenario_run
        .export
        .trace_entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            blinc_recorder::TraceEntryKind::Command(command) => Some(command.name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let playbook_commands = playbook_run
        .export
        .trace_entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            blinc_recorder::TraceEntryKind::Command(command) => Some(command.name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(scenario_commands, playbook_commands);
    assert!(scenario_run
        .export
        .trace_entries
        .iter()
        .any(|entry| matches!(entry.kind, blinc_recorder::TraceEntryKind::Assertion(_))));
    assert!(playbook_run
        .export
        .trace_entries
        .iter()
        .any(|entry| matches!(entry.kind, blinc_recorder::TraceEntryKind::Assertion(_))));
}
