# App Headless Diagnostics Tooling Implementation Plan

I'm using the writing-plans skill to create the implementation plan.

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an app-level headless execution mode focused on UI development diagnostics: goal/assertion checks, scenario-driven execution, and failure reports using recorder data.

**Architecture:** Implement a minimal headless runtime path at the app layer (not debugger) that runs without `WindowedApp::run`, collects evidence through recorder-compatible outputs, and evaluates explicit assertions. Keep windowed behavior unchanged and add only one new execution branch (`--headless`). Start with passive diagnostics + deterministic actions, then expand actions incrementally.

**Tech Stack:** Rust, `clap`, `blinc_app`, `blinc_recorder`, `serde`, `serde_json`, `anyhow`.

---

## Scope and Assumptions (Karpathy-first)

- We are building a **developer tool**, not an end-user feature.
- We target framework-level support in this repository (`blinc_app` + template/CLI wiring), not one private app codebase.
- MVP avoids platform backend rewrites (no hidden desktop window trick). It adds a true non-window branch.
- MVP action set is intentionally small (YAGNI): `wait`, `tick`, `assert_exists`, `assert_text_contains`.
- Recorder remains source of truth for exported diagnostics payloads.

## Success Criteria

- Running app with `--headless --scenario <file>` executes without creating a native window.
- Assertions evaluate deterministically and produce machine-readable failure output.
- Recorder-compatible export/report is produced for failed runs.
- Existing windowed entry path stays behaviorally unchanged.

## Worktree and Skills

- Run in a dedicated worktree (created via brainstorming flow).
- Apply `@karpathy-guidelines` during implementation/review.
- Execute this plan with `@superpowers:executing-plans` (or subagent-driven option in current session).

### Task 1: Define Headless Runtime Contract in `blinc_app`

**Files:**
- Create: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/headless_runtime.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/lib.rs`
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn headless_runtime_runs_fixed_frame_budget() {
    use crate::headless_runtime::{HeadlessRunConfig, HeadlessRuntime};

    let mut frames = 0u32;
    let cfg = HeadlessRunConfig { width: 800, height: 600, max_frames: 3, tick_ms: 16 };

    HeadlessRuntime::run(cfg, |_ctx| {
        frames += 1;
    }).expect("headless run should succeed");

    assert_eq!(frames, 3);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app headless_runtime_runs_fixed_frame_budget -- --exact`
Expected: FAIL (`headless_runtime` module/types not found).

**Step 3: Write minimal implementation**

```rust
pub struct HeadlessRunConfig { pub width: u32, pub height: u32, pub max_frames: u32, pub tick_ms: u64 }
pub struct HeadlessContext { pub frame_index: u32, pub width: u32, pub height: u32 }
pub struct HeadlessRuntime;

impl HeadlessRuntime {
    pub fn run<F>(cfg: HeadlessRunConfig, mut on_frame: F) -> anyhow::Result<()>
    where
        F: FnMut(&HeadlessContext),
    {
        for i in 0..cfg.max_frames {
            on_frame(&HeadlessContext { frame_index: i, width: cfg.width, height: cfg.height });
        }
        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app headless_runtime_runs_fixed_frame_budget -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_runtime.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/tests.rs
git commit -m "feat(blinc_app): add minimal headless runtime contract"
```

### Task 2: Add Scenario File Parsing (MVP DSL)

**Files:**
- Create: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/headless_scenario.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/lib.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/Cargo.toml`
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn parses_wait_and_assert_steps() {
    use crate::headless_scenario::{HeadlessScenario, ScenarioStep};

    let json = r#"{
      "steps": [
        {"type":"wait","ms":100},
        {"type":"assert_exists","id":"login.button"}
      ]
    }"#;

    let scenario: HeadlessScenario = serde_json::from_str(json).unwrap();
    assert!(matches!(scenario.steps[0], ScenarioStep::Wait { ms: 100 }));
    assert!(matches!(scenario.steps[1], ScenarioStep::AssertExists { .. }));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app parses_wait_and_assert_steps -- --exact`
Expected: FAIL (scenario types missing).

**Step 3: Write minimal implementation**

```rust
#[derive(serde::Deserialize)]
pub struct HeadlessScenario { pub steps: Vec<ScenarioStep> }

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScenarioStep {
    Wait { ms: u64 },
    Tick { frames: u32 },
    AssertExists { id: String },
    AssertTextContains { id: String, value: String },
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app parses_wait_and_assert_steps -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_scenario.rs crates/blinc_app/src/lib.rs crates/blinc_app/Cargo.toml crates/blinc_app/src/tests.rs
git commit -m "feat(blinc_app): add headless scenario parser"
```

### Task 3: Implement Assertion Engine for Goal Checks

**Files:**
- Create: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/headless_assert.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/lib.rs`
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn assert_text_contains_reports_failure_detail() {
    use crate::headless_assert::{AssertionResult, evaluate_assert_text_contains};

    let result = evaluate_assert_text_contains("title", "Welcome", Some("Hello"));
    assert!(matches!(result, AssertionResult::Failed { .. }));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app assert_text_contains_reports_failure_detail -- --exact`
Expected: FAIL (assert module/functions missing).

**Step 3: Write minimal implementation**

```rust
pub enum AssertionResult { Passed, Failed { code: String, message: String } }

pub fn evaluate_assert_text_contains(id: &str, expected: &str, actual: Option<&str>) -> AssertionResult {
    match actual {
        Some(text) if text.contains(expected) => AssertionResult::Passed,
        Some(text) => AssertionResult::Failed {
            code: "text_mismatch".into(),
            message: format!("{id}: expected substring '{expected}', got '{text}'"),
        },
        None => AssertionResult::Failed {
            code: "missing_element".into(),
            message: format!("{id}: element not found"),
        },
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app assert_text_contains_reports_failure_detail -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_assert.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/tests.rs
git commit -m "feat(blinc_app): add headless goal assertion engine"
```

### Task 4: Connect Headless Runtime + Scenario + Assertions into One Runner

**Files:**
- Create: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/headless_runner.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/lib.rs`
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn runner_stops_on_first_failed_assertion() {
    use crate::headless_runner::{run_scenario, RunOutcome};

    let scenario_json = r#"{
      "steps": [
        {"type":"assert_exists","id":"missing.node"},
        {"type":"tick","frames":10}
      ]
    }"#;

    let outcome = run_scenario(scenario_json).unwrap();
    assert!(matches!(outcome, RunOutcome::Failed { .. }));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app runner_stops_on_first_failed_assertion -- --exact`
Expected: FAIL (`headless_runner` not found).

**Step 3: Write minimal implementation**

```rust
pub enum RunOutcome { Passed, Failed { step_index: usize, code: String, message: String } }

pub fn run_scenario(input: &str) -> anyhow::Result<RunOutcome> {
    let scenario: HeadlessScenario = serde_json::from_str(input)?;
    for (i, step) in scenario.steps.iter().enumerate() {
        if let Some(failure) = evaluate_step(step) {
            return Ok(RunOutcome::Failed { step_index: i, code: failure.code, message: failure.message });
        }
    }
    Ok(RunOutcome::Passed)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app runner_stops_on_first_failed_assertion -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_runner.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/tests.rs
git commit -m "feat(blinc_app): wire headless diagnostics runner"
```

### Task 5: Add App Entry `--headless` Mode in Generated Rust Template

**Files:**
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_cli/src/project.rs`
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_cli/src/project.rs` (template generation tests)
- Docs: `/Users/cypark/Documents/project/Blinc/crates/blinc_cli/README.md` (if present; else use root CLI docs)

**Step 1: Write the failing test**

```rust
#[test]
fn rust_template_contains_headless_flag_branch() {
    let src = template_default("DemoApp");
    assert!(src.contains("--headless"));
    assert!(src.contains("run_headless"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_cli rust_template_contains_headless_flag_branch -- --exact`
Expected: FAIL (template does not expose headless branch).

**Step 3: Write minimal implementation**

```rust
// In generated app main:
// - parse `--headless` and optional `--scenario <path>`
// - if headless: call diagnostics runner and exit with non-zero on failure
// - else: keep existing WindowedApp path unchanged
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_cli rust_template_contains_headless_flag_branch -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/project.rs crates/blinc_cli/README.md
git commit -m "feat(blinc_cli): scaffold app-level headless diagnostics mode"
```

### Task 6: Recorder-Compatible Failure Report Output

**Files:**
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/headless_runner.rs`
- Create: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/headless_report.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/lib.rs`
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn failed_run_writes_machine_readable_report() {
    use crate::headless_report::HeadlessReport;

    let report = HeadlessReport::failed("assert_exists", 0, "missing.node");
    let json = serde_json::to_string(&report).unwrap();

    assert!(json.contains("\"status\":\"failed\""));
    assert!(json.contains("\"assert_exists\""));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app failed_run_writes_machine_readable_report -- --exact`
Expected: FAIL (report type missing).

**Step 3: Write minimal implementation**

```rust
#[derive(serde::Serialize)]
pub struct HeadlessReport {
    pub status: String,
    pub failed_step_index: Option<usize>,
    pub assertion: Option<String>,
    pub message: Option<String>,
}
```

- Emit JSON report to `stdout` by default.
- If output path is provided, write report JSON to file.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app failed_run_writes_machine_readable_report -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_runner.rs crates/blinc_app/src/headless_report.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/tests.rs
git commit -m "feat(blinc_app): emit headless diagnostics failure reports"
```

### Task 7: Documentation + Developer Workflow

**Files:**
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/README.md`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_recorder/README.md`
- Create: `/Users/cypark/Documents/project/Blinc/docs/headless-diagnostics.md`

**Step 1: Write the failing doc check**

Run: `rg -n "headless diagnostics|--headless|scenario|assert" /Users/cypark/Documents/project/Blinc/crates/blinc_app/README.md /Users/cypark/Documents/project/Blinc/docs/headless-diagnostics.md`
Expected: FAIL for missing new guide file and missing sections.

**Step 2: Add minimal docs**

Include:
- What problem this solves (UI dev goal/test/debug loop)
- CLI usage examples
- Scenario JSON examples
- Exit code behavior for CI integration

**Step 3: Verify docs presence**

Run: `rg -n "headless diagnostics|--headless|assert_exists|assert_text_contains" /Users/cypark/Documents/project/Blinc/crates/blinc_app/README.md /Users/cypark/Documents/project/Blinc/docs/headless-diagnostics.md`
Expected: PASS.

**Step 4: Formatting gate**

Run:
- `cargo fmt --all`
- `cargo fmt --all -- --check`

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/README.md crates/blinc_recorder/README.md docs/headless-diagnostics.md
git commit -m "docs: add app headless diagnostics workflow guide"
```

## Final Verification Checklist

- `cargo test -p blinc_app headless_ -- --nocapture`
- `cargo test -p blinc_cli rust_template_contains_headless_flag_branch -- --exact`
- `cargo fmt --all`
- `cargo fmt --all -- --check`

Expected: all PASS.
