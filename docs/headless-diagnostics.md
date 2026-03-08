# Headless Diagnostics Workflow

This document is the real end-to-end workflow for the agent-debugging toolchain.
It is based on the same generated-app flow that is exercised in repository tests and
manually re-run during development:

- generate a real Blinc app
- drive it with `click`, `fill`, `press`, and `scroll`
- write machine-readable reports
- export snapshot/trace artifacts
- open the trace in `blinc_debugger`

If you are running from source instead of an installed CLI, replace `blinc ...` with:

```bash
cargo run -p blinc_cli -- ...
```

Rust project scaffolding still depends on a local Blinc checkout. If you are using an installed
`blinc` binary, run these steps from a Blinc source tree or export `BLINC_PATH=/path/to/Blinc`
before `blinc new --rust ...`.

If you are running the debugger from source, replace `blinc-debugger ...` with:

```bash
cargo run -p blinc_debugger -- ...
```

## Validated E2E: Generated Counter App

This is the shortest end-to-end path that proves the full stack works:

1. Create a real Rust app from the built-in counter template.
2. Run a headless scenario against the generated binary.
3. Run the same logical flow through a playbook.
4. Run the same scenario through the desktop harness.
5. Export a trace and open it in the debugger.

### 1) Create a disposable workspace

```bash
workdir="$(mktemp -d /tmp/blinc-e2e-XXXXXX)"
export WORKDIR="$workdir"
cd "$workdir"
blinc new E2EApp --rust -t counter
```

This creates a real app at `"$workdir/E2EApp"`.

### 2) Write a minimal scenario

Create `scenario.json` next to the project:

```json
{
  "name": "counter-smoke",
  "steps": [
    { "type": "click", "id": "counter.increment" },
    { "type": "assert_text_contains", "id": "counter.value", "value": "Count: 1" }
  ]
}
```

Run it headlessly:

```bash
blinc automation run "$workdir/E2EApp" \
  --scenario "$workdir/scenario.json" \
  --report "$workdir/headless-scenario-report.json"
```

Expected result:

- process exit code `0`
- report file exists
- report JSON contains `"status": "passed"`

Quick check:

```bash
python3 - <<'PY'
import json, pathlib, os
workdir = pathlib.Path(os.environ["WORKDIR"])
report = json.loads((workdir / "headless-scenario-report.json").read_text())
print(report["status"])
PY
```

### 3) Run the same behavior through a playbook

Create `playbook.yaml`:

```yaml
initial_state: idle
states:
  - clicked
transitions:
  - name: increment
    from: idle
    event: click
    to: clicked
    steps:
      - type: click
        id: counter.increment
      - type: assert_text_contains
        id: counter.value
        value: "Count: 1"
```

Run it:

```bash
blinc automation run "$workdir/E2EApp" \
  --playbook "$workdir/playbook.yaml" \
  --report "$workdir/playbook-report.json"
```

Expected result:

- process exit code `0`
- report JSON contains `"status": "passed"`

### 4) Run the deterministic desktop harness path

Run the same scenario through the desktop harness:

```bash
blinc automation run "$workdir/E2EApp" \
  --desktop-harness \
  --scenario "$workdir/scenario.json" \
  --report "$workdir/desktop-report.json"
```

Expected result:

- process exit code `0`
- report JSON contains `"status": "passed"`

This is the parity check that matters for actual tool usage:

- headless scenario passes
- playbook passes
- desktop harness passes

### 5) Export a snapshot and trace artifact

Create `trace-scenario.json`:

```json
{
  "name": "counter-with-artifacts",
  "steps": [
    { "type": "click", "id": "counter.increment" },
    { "type": "snapshot", "path": "/tmp/blinc-e2e-snapshot.json" },
    { "type": "export_trace", "path": "/tmp/blinc-e2e-trace.json" },
    { "type": "assert_text_contains", "id": "counter.value", "value": "Count: 1" }
  ]
}
```

Run it:

```bash
blinc automation run "$workdir/E2EApp" \
  --scenario "$workdir/trace-scenario.json" \
  --report "$workdir/headless-artifact-report.json"
```

Expected result:

- report JSON contains `"status": "passed"`
- snapshot file exists
- trace file exists

Example validation:

```bash
python3 - <<'PY'
import json, pathlib
trace = json.loads(pathlib.Path("/tmp/blinc-e2e-trace.json").read_text())
print("trace_entries =", len(trace["trace_entries"]))
print("snapshots =", len(trace["snapshots"]))
PY
```

The trace is the same `RecordingExport` container used by `blinc_debugger`.

### 6) Open the exported trace in the debugger

```bash
blinc-debugger /tmp/blinc-e2e-trace.json
```

Inside the debugger, verify:

- the timeline shows command and assertion markers
- the command panel shows the `click` and assertion sequence
- the evidence panel shows exported artifacts and assertion outcomes
- the inspector can resolve `counter.value` in the snapshot tree
- the inspector shows semantic metadata when the selected element exposes accessibility info
- the inspector shows `ViewModel State` entries captured from keyed runtime state

## What This E2E Covers

This workflow validates the actual user-facing path, not just library internals:

- `blinc new --rust -t counter`
- generated app automation entrypoint
- `blinc automation run --scenario`
- `blinc automation run --playbook`
- `blinc automation run --desktop-harness`
- snapshot export
- trace export
- `blinc_debugger` trace inspection

## Scenario Semantics

The current scenario DSL supports:

- `click`
- `fill`
- `press`
- `scroll`
- `snapshot`
- `export_trace`
- `assert_exists`
- `assert_text_contains`

Example login scenario:

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

Important interaction rules for automation:

- locator-based `click` fails if an active overlay/backdrop would consume the interaction
- `click_at` may dismiss overlays because it models raw coordinate input
- `scroll` backdrop blocking is intentionally limited to blocking overlays
- `fill` and `press` respect the same overlay occlusion rules as targeted clicks

## App-Level API Surface

If you want to drive the runtime directly from Rust instead of the CLI, use the same app-backed path:

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

Playbooks compile onto the existing FSM runtime and execute through the same automation session:

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

The returned `AutomationRun` contains:

- `report`: pass/fail summary for CI or local automation
- `export`: `RecordingExport` with events, snapshots, and trace entries

## Legacy Probe Path

The older probe-driven APIs still exist for assertion-only diagnostics:

- `run_loaded_scenario_with_probe(...)`
- `run_loaded_scenario_with_owned_probe(...)`

Use them when you want to assert against a synthetic or domain-level snapshot instead of driving the real UI runtime.

## CI Recommendation

For CI or agent loops:

- fail the job on non-zero exit
- always persist the JSON report
- persist exported trace/snapshot artifacts on both pass and fail when possible
- open the trace in `blinc_debugger` for forensic inspection when a run fails
