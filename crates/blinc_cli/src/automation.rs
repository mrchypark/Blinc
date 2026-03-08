use anyhow::{bail, Context, Result};
use blinc_app::Playbook;
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand, Debug)]
pub enum AutomationCommands {
    /// Run an automation scenario or playbook against a Blinc app
    Run(AutomationRunArgs),
    /// Validate a state-machine playbook
    Validate(AutomationValidateArgs),
    /// Export a simple SVG diagram for a playbook
    ExportDiagram(AutomationExportDiagramArgs),
}

#[derive(Args, Debug)]
pub struct AutomationRunArgs {
    /// Project directory containing the app Cargo manifest
    #[arg(default_value = ".")]
    pub source: String,

    /// Cargo binary target to run when the project defines multiple binaries
    #[arg(long)]
    pub bin: Option<String>,

    /// Scenario JSON file to execute; relative paths resolve from the caller's current directory
    #[arg(long, group = "artifact")]
    pub scenario: Option<String>,

    /// Playbook YAML file to execute; relative paths resolve from the caller's current directory
    #[arg(long, group = "artifact")]
    pub playbook: Option<String>,

    /// Write a machine-readable report to this path; relative paths resolve from the caller's current directory
    #[arg(long)]
    pub report: Option<String>,

    /// Force the headless automation runner (default when omitted)
    #[arg(long, conflicts_with = "desktop_harness")]
    pub headless: bool,

    /// Drive the desktop harness path instead of the headless runner
    #[arg(long, conflicts_with = "headless")]
    pub desktop_harness: bool,
}

#[derive(Args, Debug)]
pub struct AutomationValidateArgs {
    /// Playbook YAML file to validate
    #[arg(long)]
    pub playbook: String,
}

#[derive(Args, Debug)]
pub struct AutomationExportDiagramArgs {
    /// Playbook YAML file to visualize
    #[arg(long)]
    pub playbook: String,

    /// Output SVG path
    #[arg(short, long)]
    pub output: String,
}

pub fn cmd_automation(command: AutomationCommands) -> Result<()> {
    match command {
        AutomationCommands::Run(args) => cmd_run(args),
        AutomationCommands::Validate(args) => cmd_validate(args),
        AutomationCommands::ExportDiagram(args) => cmd_export_diagram(args),
    }
}

fn cmd_run(args: AutomationRunArgs) -> Result<()> {
    let invocation_cwd =
        std::env::current_dir().context("failed to determine automation invocation directory")?;
    let source = std::fs::canonicalize(Path::new(&args.source))
        .with_context(|| format!("failed to resolve automation source {}", args.source))?;
    let manifest = source.join("Cargo.toml");
    if !manifest.is_file() {
        bail!("no Cargo.toml found under {}", source.display());
    }

    let mut command = build_run_command(&invocation_cwd, &source, &manifest, &args)?;
    let command_desc = format!("{command:?}");

    let status = command
        .status()
        .with_context(|| format!("failed to launch automation run in {}", source.display()))?;

    if !status.success() {
        bail!("automation run failed with status {status}: {command_desc}");
    }

    Ok(())
}

fn build_run_command(
    invocation_cwd: &Path,
    source: &Path,
    manifest: &Path,
    args: &AutomationRunArgs,
) -> Result<Command> {
    let artifact = resolve_run_artifact(args, invocation_cwd)?;
    let mut command = Command::new("cargo");
    command.current_dir(source);
    command
        .arg("run")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--features")
        .arg("desktop");
    command.env("BLINC_AUTOMATION_INVOCATION_CWD", invocation_cwd);

    if let Some(bin) = args.bin.as_deref() {
        command.arg("--bin").arg(bin);
    }

    command.arg("--");
    if args.desktop_harness {
        command.arg("--desktop-harness");
    } else if args.headless {
        command.arg("--headless");
    } else {
        // Default to the headless runner when automation is invoked through the CLI.
        command.arg("--headless");
    }
    artifact.append_args(&mut command);

    if let Some(report) = args.report.as_deref() {
        command
            .arg("--report")
            .arg(resolve_invocation_path(invocation_cwd, report));
    }
    Ok(command)
}

fn cmd_validate(args: AutomationValidateArgs) -> Result<()> {
    let playbook = Playbook::from_path(Path::new(&args.playbook))?;
    let compiled = playbook.compile()?;
    let _ = compiled.execution_scenario()?;
    Ok(())
}

fn cmd_export_diagram(args: AutomationExportDiagramArgs) -> Result<()> {
    let playbook = Playbook::from_path(Path::new(&args.playbook))?;
    let svg = playbook_svg(&playbook)?;
    let output = Path::new(&args.output);
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(output, svg)
        .with_context(|| format!("failed to write exported diagram to {}", output.display()))?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RunArtifact {
    Scenario(PathBuf),
    Playbook(PathBuf),
}

impl RunArtifact {
    fn append_args(&self, command: &mut Command) {
        match self {
            RunArtifact::Scenario(path) => {
                command.arg("--scenario").arg(path);
            }
            RunArtifact::Playbook(path) => {
                command.arg("--playbook").arg(path);
            }
        }
    }
}

fn resolve_run_artifact(args: &AutomationRunArgs, invocation_cwd: &Path) -> Result<RunArtifact> {
    match (args.scenario.as_deref(), args.playbook.as_deref()) {
        (Some(path), None) => Ok(RunArtifact::Scenario(resolve_invocation_path(
            invocation_cwd,
            path,
        ))),
        (None, Some(path)) => Ok(RunArtifact::Playbook(resolve_invocation_path(
            invocation_cwd,
            path,
        ))),
        (Some(_), Some(_)) => {
            bail!("automation run accepts either --scenario or --playbook, not both")
        }
        (None, None) => bail!("automation run requires either --scenario or --playbook"),
    }
}

fn resolve_invocation_path(invocation_cwd: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        invocation_cwd.join(path)
    }
}

fn playbook_svg(playbook: &Playbook) -> Result<String> {
    let compiled = playbook.compile()?;
    let mut states = compiled.state_ids.iter().collect::<Vec<_>>();
    states.sort_by_key(|(_, state_id)| **state_id);

    let node_width = 168.0f32;
    let node_height = 56.0f32;
    let x_spacing = 220.0f32;
    let start_x = 72.0f32;
    let start_y = 132.0f32;
    let width =
        (start_x * 2.0 + ((states.len().saturating_sub(1)) as f32 * x_spacing) + node_width)
            .max(420.0);
    let height = 320.0f32;

    let mut coords = std::collections::HashMap::new();
    for (index, (name, _)) in states.iter().enumerate() {
        coords.insert(
            (*name).clone(),
            (
                start_x + (index as f32 * x_spacing),
                start_y,
                start_x + (index as f32 * x_spacing) + node_width * 0.5,
                start_y + node_height * 0.5,
            ),
        );
    }

    let mut edges = String::new();
    let mut labels = String::new();
    for transition in &playbook.transitions {
        let Some((from_x, from_y, from_cx, from_cy)) = coords.get(&transition.from) else {
            continue;
        };
        let Some((to_x, _to_y, to_cx, to_cy)) = coords.get(&transition.to) else {
            continue;
        };

        if transition.from == transition.to {
            let loop_top = from_y - 42.0;
            let loop_right = from_x + node_width + 18.0;
            edges.push_str(&format!(
                r##"<path d="M {x2:.1} {y_mid:.1} C {loop_right:.1} {y_mid:.1}, {loop_right:.1} {loop_top:.1}, {cx:.1} {loop_top:.1} C {x1:.1} {loop_top:.1}, {x1:.1} {y_mid:.1}, {x1:.1} {y_mid:.1}" stroke="#38bdf8" stroke-width="2.5" fill="none" marker-end="url(#arrowhead)"/>"##,
                x1 = from_x + 12.0,
                x2 = from_x + node_width - 12.0,
                y_mid = from_cy,
                loop_right = loop_right,
                loop_top = loop_top,
                cx = from_cx
            ));
            labels.push_str(&format!(
                r##"<text x="{x:.1}" y="{y:.1}" fill="#cbd5e1" font-family="Menlo, Monaco, monospace" font-size="12" text-anchor="middle">{label}</text>"##,
                x = from_cx,
                y = loop_top - 10.0,
                label = escape_xml(&transition.event)
            ));
            continue;
        }

        let start = if from_cx < to_cx {
            (from_x + node_width, from_cy)
        } else {
            (*from_x, from_cy)
        };
        let end = if from_cx < to_cx {
            (*to_x, to_cy)
        } else {
            (to_x + node_width, to_cy)
        };
        let label_x = (start.0 + end.0) * 0.5;
        let label_y = (start.1 + end.1) * 0.5 - 12.0;
        edges.push_str(&format!(
            r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="#38bdf8" stroke-width="2.5" marker-end="url(#arrowhead)"/>"##,
            x1 = start.0,
            y1 = start.1,
            x2 = end.0,
            y2 = end.1
        ));
        labels.push_str(&format!(
            r##"<text x="{x:.1}" y="{y:.1}" fill="#cbd5e1" font-family="Menlo, Monaco, monospace" font-size="12" text-anchor="middle">{label}</text>"##,
            x = label_x,
            y = label_y,
            label = escape_xml(&transition.event)
        ));
    }

    let mut nodes = String::new();
    for (name, state_id) in states {
        let (x, y, cx, cy) = coords
            .get(name)
            .copied()
            .expect("state coordinates should exist");
        let is_initial = *state_id == compiled.initial_state;
        nodes.push_str(&format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="18" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}"/>"#,
            x = x,
            y = y,
            w = node_width,
            h = node_height,
            fill = if is_initial { "#0f766e" } else { "#111827" },
            stroke = if is_initial { "#99f6e4" } else { "#334155" },
            stroke_width = if is_initial { 3 } else { 2 }
        ));
        nodes.push_str(&format!(
            r##"<text x="{x:.1}" y="{y:.1}" fill="#f8fafc" font-family="Menlo, Monaco, monospace" font-size="15" text-anchor="middle" dominant-baseline="middle">{label}</text>"##,
            x = cx,
            y = cy,
            label = escape_xml(name)
        ));
    }

    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0}" height="{height:.0}" viewBox="0 0 {width:.0} {height:.0}">
  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="#38bdf8"/>
    </marker>
  </defs>
  <rect width="{width:.0}" height="{height:.0}" fill="#020617"/>
  <text x="{title_x:.1}" y="44" fill="#e2e8f0" font-family="Menlo, Monaco, monospace" font-size="20" text-anchor="middle">Blinc Playbook Diagram</text>
  {edges}
  {labels}
  {nodes}
</svg>"##,
        width = width,
        height = height,
        title_x = width * 0.5,
        edges = edges,
        labels = labels,
        nodes = nodes
    ))
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::{build_run_command, playbook_svg, AutomationCommands, AutomationRunArgs};
    use crate::project::create_rust_project;
    use blinc_app::Playbook;
    use clap::Parser;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Parser)]
    struct Harness {
        #[command(subcommand)]
        command: AutomationCommands,
    }

    fn cwd_guard() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().expect("cwd test lock")
    }

    #[test]
    fn cli_parses_automation_run_command() {
        let parsed = Harness::try_parse_from([
            "blinc",
            "run",
            ".",
            "--bin",
            "demoapp_desktop",
            "--scenario",
            "scenario.json",
            "--report",
            "report.json",
        ])
        .expect("automation run args should parse");

        match parsed.command {
            AutomationCommands::Run(args) => {
                assert_eq!(args.source, ".");
                assert_eq!(args.bin.as_deref(), Some("demoapp_desktop"));
                assert_eq!(args.scenario.as_deref(), Some("scenario.json"));
                assert_eq!(args.playbook, None);
                assert_eq!(args.report.as_deref(), Some("report.json"));
                assert!(!args.headless);
                assert!(!args.desktop_harness);
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn cli_parses_automation_run_playbook_command() {
        let parsed = Harness::try_parse_from([
            "blinc",
            "run",
            ".",
            "--desktop-harness",
            "--playbook",
            "login.yaml",
        ])
        .expect("automation playbook args should parse");

        match parsed.command {
            AutomationCommands::Run(args) => {
                assert_eq!(args.playbook.as_deref(), Some("login.yaml"));
                assert!(args.desktop_harness);
                assert!(!args.headless);
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn cli_parses_playbook_validate_command() {
        let parsed = Harness::try_parse_from(["blinc", "validate", "--playbook", "login.yaml"])
            .expect("validate args should parse");

        match parsed.command {
            AutomationCommands::Validate(args) => {
                assert_eq!(args.playbook, "login.yaml");
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn cli_parses_playbook_export_diagram_command() {
        let parsed = Harness::try_parse_from([
            "blinc",
            "export-diagram",
            "--playbook",
            "login.yaml",
            "--output",
            "diagram.svg",
        ])
        .expect("export-diagram args should parse");

        match parsed.command {
            AutomationCommands::ExportDiagram(args) => {
                assert_eq!(args.playbook, "login.yaml");
                assert_eq!(args.output, "diagram.svg");
            }
            _ => panic!("expected export-diagram command"),
        }
    }

    #[test]
    fn playbook_svg_renders_state_nodes_and_transition_labels() {
        let playbook = Playbook::from_yaml(
            r#"
initial_state: idle
states: [submitted]
transitions:
  - from: idle
    event: submit
    to: submitted
"#,
        )
        .expect("playbook should parse");

        let svg = playbook_svg(&playbook).expect("svg export should succeed");

        assert!(svg.contains("Blinc Playbook Diagram"));
        assert!(svg.contains("idle"));
        assert!(svg.contains("submitted"));
        assert!(svg.contains("submit"));
        assert!(svg.contains("marker-end=\"url(#arrowhead)\""));
    }

    #[test]
    fn cmd_validate_rejects_ambiguous_playbook_execution_order() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("blinc-automation-validate-{nonce}.yaml"));
        std::fs::write(
            &path,
            r#"
initial_state: idle
states: [ready, cancelled]
transitions:
  - from: idle
    event: submit
    to: ready
    steps:
      - type: tick
        frames: 1
  - from: idle
    event: cancel
    to: cancelled
    steps:
      - type: tick
        frames: 1
"#,
        )
        .expect("playbook fixture should be written");

        let err = super::cmd_validate(super::AutomationValidateArgs {
            playbook: path.display().to_string(),
        })
        .expect_err("ambiguous playbook should fail validation");

        assert!(
            err.to_string().contains("ambiguous"),
            "unexpected error: {err}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cmd_validate_rejects_invalid_embedded_scenario_steps() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("blinc-automation-invalid-step-{nonce}.yaml"));
        std::fs::write(
            &path,
            r#"
initial_state: idle
states: [ready]
transitions:
  - from: idle
    event: submit
    to: ready
    steps:
      - type: click
        id: login.submit
        role: button
"#,
        )
        .expect("playbook fixture should be written");

        let err = super::cmd_validate(super::AutomationValidateArgs {
            playbook: path.display().to_string(),
        })
        .expect_err("invalid embedded scenario step should fail validation");

        assert!(
            err.to_string()
                .contains("cannot mix id with semantic locator fields"),
            "unexpected error: {err}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn build_run_command_uses_absolute_manifest_path_for_relative_source() {
        let root = project_root();
        let relative_source = PathBuf::from("crates/blinc_cli");
        let source = root.join(&relative_source);
        let manifest = source.join("Cargo.toml");
        let command = build_run_command(
            &root,
            &source,
            &manifest,
            &AutomationRunArgs {
                source: relative_source.display().to_string(),
                bin: Some("blinc".to_string()),
                scenario: Some("scenario.json".to_string()),
                playbook: None,
                report: Some("report.json".to_string()),
                headless: false,
                desktop_harness: false,
            },
        )
        .expect("command should build");
        let args = command.get_args().collect::<Vec<_>>();
        let manifest_idx = args
            .iter()
            .position(|arg| *arg == std::ffi::OsStr::new("--manifest-path"))
            .expect("manifest path flag should exist");
        let manifest_arg = args
            .get(manifest_idx + 1)
            .expect("manifest path value should exist");
        assert!(
            Path::new(manifest_arg).is_absolute(),
            "manifest path should be absolute: {:?}",
            manifest_arg
        );
        assert!(
            args.iter().any(|arg| {
                Path::new(arg) == root.join("scenario.json")
                    || Path::new(arg) == root.join("report.json")
            }),
            "expected relative artifact paths to resolve against invocation cwd: {args:?}"
        );
    }

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root should exist")
            .to_path_buf()
    }

    #[test]
    fn build_run_command_includes_selected_bin_target() {
        let source = Path::new("/tmp/example-app");
        let manifest = source.join("Cargo.toml");
        let args = super::AutomationRunArgs {
            source: source.display().to_string(),
            bin: Some("example_desktop".to_string()),
            scenario: Some("scenario.json".to_string()),
            playbook: None,
            report: Some("report.json".to_string()),
            headless: true,
            desktop_harness: false,
        };

        let command = super::build_run_command(Path::new("/tmp"), source, &manifest, &args)
            .expect("command should build");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|pair| pair[0] == "--bin" && pair[1] == "example_desktop"),
            "expected --bin example_desktop in cargo run args: {args:?}"
        );
        assert!(
            args.windows(2)
                .any(|pair| { pair[0] == "--scenario" && pair[1] == "/tmp/scenario.json" }),
            "expected scenario path to resolve against invocation cwd: {args:?}"
        );
        assert!(
            args.windows(2)
                .any(|pair| { pair[0] == "--report" && pair[1] == "/tmp/report.json" }),
            "expected report path to resolve against invocation cwd: {args:?}"
        );
    }

    #[test]
    fn cmd_run_resolves_artifact_and_report_paths_from_invocation_directory() {
        let _guard = cwd_guard();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let workspace = std::env::temp_dir().join(format!("blinc_cli_cmd_run_{nonce}"));
        let source = workspace.join("DemoApp");

        create_rust_project(&source, "DemoApp", "com.example")
            .expect("rust project template should be generated");

        let playbook_path = workspace.join("login.yaml");
        std::fs::write(
            &playbook_path,
            r#"
initial_state: idle
states: [updated]
transitions:
  - from: idle
    event: increment
    to: updated
    steps:
      - type: click
        id: counter.increment
      - type: assert_text_contains
        id: counter.value
        value: "Count: 1"
"#,
        )
        .expect("playbook fixture should be written");

        let previous_cwd = std::env::current_dir().expect("current dir should resolve");
        std::env::set_current_dir(&workspace).expect("workspace cwd should be set");

        let result = super::cmd_run(super::AutomationRunArgs {
            source: "DemoApp".to_string(),
            bin: Some("demoapp_desktop".to_string()),
            scenario: None,
            playbook: Some("login.yaml".to_string()),
            report: Some("reports/cli-run-report.json".to_string()),
            headless: false,
            desktop_harness: false,
        });

        std::env::set_current_dir(&previous_cwd).expect("cwd should be restored");
        result.expect("automation run should succeed");

        let report_path = workspace.join("reports/cli-run-report.json");
        let report = std::fs::read_to_string(&report_path).expect("report should be written");
        let report: serde_json::Value =
            serde_json::from_str(&report).expect("report should be valid json");
        assert_eq!(
            report.get("status").and_then(|status| status.as_str()),
            Some("passed"),
            "expected passing report"
        );

        let _ = std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn cmd_run_supports_desktop_harness_with_relative_artifacts() {
        let _guard = cwd_guard();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let workspace = std::env::temp_dir().join(format!("blinc_cli_cmd_run_desktop_{nonce}"));
        let source = workspace.join("DemoApp");

        create_rust_project(&source, "DemoApp", "com.example")
            .expect("rust project template should be generated");

        let scenario_path = workspace.join("scenario.json");
        std::fs::write(
            &scenario_path,
            r#"{
  "steps": [
    {"type":"click","id":"counter.increment"},
    {"type":"snapshot","path":"artifacts/tree.json"},
    {"type":"export_trace","path":"artifacts/trace.json"},
    {"type":"assert_text_contains","id":"counter.value","value":"Count: 1"}
  ]
}"#,
        )
        .expect("scenario fixture should be written");

        let previous_cwd = std::env::current_dir().expect("current dir should resolve");
        std::env::set_current_dir(&workspace).expect("workspace cwd should be set");

        let result = super::cmd_run(super::AutomationRunArgs {
            source: "DemoApp".to_string(),
            bin: Some("demoapp_desktop".to_string()),
            scenario: Some("scenario.json".to_string()),
            playbook: None,
            report: Some("reports/cli-desktop-report.json".to_string()),
            headless: false,
            desktop_harness: true,
        });

        std::env::set_current_dir(&previous_cwd).expect("cwd should be restored");
        result.expect("desktop harness automation run should succeed");

        let report_path = workspace.join("reports/cli-desktop-report.json");
        let report = std::fs::read_to_string(&report_path).expect("report should be written");
        let report: serde_json::Value =
            serde_json::from_str(&report).expect("report should be valid json");
        assert_eq!(
            report.get("status").and_then(|status| status.as_str()),
            Some("passed"),
            "expected passing desktop harness report"
        );

        let snapshot_path = workspace.join("artifacts/tree.json");
        let snapshot =
            std::fs::read_to_string(&snapshot_path).expect("snapshot artifact should be written");
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot).expect("snapshot should be valid json");
        assert!(
            snapshot
                .get("root_id")
                .and_then(|root| root.as_str())
                .is_some_and(|root| !root.is_empty()),
            "expected snapshot artifact to include a non-empty root id"
        );
        assert!(
            snapshot
                .get("elements")
                .and_then(|elements| elements.as_object())
                .is_some_and(|elements| elements.contains_key("counter.value")),
            "expected counter.value to appear in the snapshot artifact"
        );

        let trace_path = workspace.join("artifacts/trace.json");
        let trace = std::fs::read_to_string(&trace_path).expect("trace artifact should be written");
        let trace: serde_json::Value =
            serde_json::from_str(&trace).expect("trace should be valid json");
        assert!(
            trace
                .get("trace_entries")
                .and_then(|entries| entries.as_array())
                .is_some_and(|entries| !entries.is_empty()),
            "expected trace artifact to contain entries"
        );

        let _ = std::fs::remove_dir_all(&workspace);
    }
}
