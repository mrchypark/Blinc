# Headless Diagnostics Workflow

This workflow is for UI development tooling where you want to:

- define explicit goals/assertions,
- inject real UI events (`click`, `fill`, `press`, `scroll`) against stable IDs,
- run deterministic scenarios without opening a native window,
- capture machine-readable failure reports,
- export the same trace container that `blinc_debugger` can inspect later.

## 1) Define scenario

Example `scenario.json`:

```json
{
  "steps": [
    { "type": "fill", "id": "login.email", "value": "person@example.com" },
    { "type": "click", "id": "login.submit" },
    { "type": "assert_text_contains", "id": "login.status", "value": "Signed in" },
    { "type": "export_trace", "path": "target/automation/login-trace.json" }
  ]
}
```

## 2) Execute the app-backed runner

Use `run_headless_scenario(...)` when you want the real app UI to be built, driven, and traced:

```rust
use blinc_app::prelude::*;

let scenario = HeadlessScenario::from_path("scenario.json".as_ref())?;
let run = run_headless_scenario(
    HeadlessRunConfig::default(),
    &scenario,
    |ctx| app_ui(ctx),
)?;

run.report.write_to_writer(&mut std::io::stdout())?;
# Ok::<(), anyhow::Error>(())
```

The returned `AutomationRun` contains:

- `report`: pass/fail summary for CI or local runs
- `export`: `RecordingExport` with events, snapshots, and trace entries

## 3) Optional: validate or execute a playbook

`blinc_app::Playbook` compiles state-machine YAML onto the existing FSM runtime, then flattens into the same automation execution path:

```yaml
initial_state: idle
states: [filling, submitted]
transitions:
  - from: idle
    event: fill
    to: filling
    steps:
      - type: fill
        id: login.email
        value: person@example.com
```

You can execute that playbook through the same runtime surface:

```rust
use blinc_app::prelude::*;

let playbook = Playbook::from_path("login.yaml".as_ref())?;
let run = run_headless_playbook(
    HeadlessRunConfig::default(),
    &playbook,
    |ctx| app_ui(ctx),
)?;
# let _ = run;
# Ok::<(), anyhow::Error>(())
```

Generated Rust app templates and `blinc automation run` also accept either:

- `--scenario scenario.json`
- `--playbook login.yaml`

When an automation artifact is provided without an explicit mode flag, they default to the headless runner. Use `--desktop-harness` to force the deterministic desktop harness path.
Relative `--scenario`, `--playbook`, and `--report` paths follow normal CLI semantics: they resolve from the caller's current working directory. `blinc automation run` normalizes those paths to absolute paths before it changes into the app source directory, so the generated app receives the same file locations you named at the call site.

## 4) Legacy probe path

The older probe-driven APIs still exist for assertion-only diagnostics:

- `run_loaded_scenario_with_probe(...)`
- `run_loaded_scenario_with_owned_probe(...)`

Use them when you want to assert against a synthetic or domain-level snapshot instead of driving the real UI runtime.

## Exit behavior recommendation

- exit `0` on pass,
- exit non-zero on failure,
- persist JSON report and exported trace as CI artifacts,
- open the exported trace in `blinc_debugger` for forensic inspection.
