# Agent Debugging Platform Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend Blinc's existing recorder, debugger, selector, FSM, and headless diagnostics infrastructure into a complete agent-debugging platform for desktop and headless app development.

**Architecture:** Reuse the current `blinc_recorder`, `blinc_app`, `blinc_layout`, `blinc_core`, `blinc_cli`, and `blinc_debugger` layers instead of creating parallel foundational crates. Grow one shared command-and-trace model by extending `RecordingExport`, enriching selector resolution, upgrading the current headless scenario runner, and teaching the debugger to inspect the resulting evidence.

**Tech Stack:** Rust 2021, Cargo workspace crates, `serde`, `serde_json`, `serde_yaml`, `anyhow`, `thiserror`, existing Blinc runtime crates and developer-tooling modules.

---

## Global Constraints / Guardrails

- Reuse existing modules first. Do not add a new foundational crate unless the existing home is proven unworkable.
- Follow TDD for every behavior: write the failing test first, confirm the failure, then add the minimum code to pass.
- Keep existing recorder/debugger/headless diagnostics workflows working while enriching them.
- Desktop plus headless are in scope; mobile is out of scope in this plan.
- Build state-machine playbooks on top of the existing FSM runtime.
- Build semantic locators on top of the existing selector registry.

### Task 1: Extend `RecordingExport` into the canonical trace container

**Files:**
- Modify: `crates/blinc_recorder/src/session/recording.rs`
- Modify: `crates/blinc_recorder/src/lib.rs`
- Create: `crates/blinc_recorder/src/trace.rs`
- Test: `crates/blinc_recorder/src/session/recording.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn recording_export_round_trips_with_command_and_assertion_entries() {}

#[test]
fn trace_entries_keep_monotonic_sequence_numbers() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_recorder recording_export_round_trips_with_command_and_assertion_entries -- --nocapture`
Expected: FAIL because `RecordingExport` only carries `events`, `snapshots`, and `stats`.

**Step 3: Write minimal implementation**

Add trace-oriented types inside `blinc_recorder`, such as:

- `TraceEntry`
- `TraceCommandRecord`
- `TraceLocatorResolution`
- `TraceAssertionRecord`
- `TraceArtifactRecord`

Extend `RecordingExport` with a new optional or defaulted trace entry collection while keeping the existing `events`, `snapshots`, and `stats` fields intact.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_recorder recording_export_round_trips_with_command_and_assertion_entries -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_recorder/src/session/recording.rs crates/blinc_recorder/src/lib.rs crates/blinc_recorder/src/trace.rs
git commit -m "feat: extend recording export with trace-oriented entries"
```

### Task 2: Reuse recorder snapshots in app-level diagnostics instead of the current minimal snapshot model

**Files:**
- Modify: `crates/blinc_app/src/headless_assert.rs`
- Modify: `crates/blinc_app/src/headless_runner.rs`
- Modify: `crates/blinc_app/src/lib.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn headless_asserts_can_read_text_from_tree_snapshot_backed_probe() {}

#[test]
fn missing_element_failure_uses_tree_snapshot_ids() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app headless_asserts_can_read_text_from_tree_snapshot_backed_probe -- --exact`
Expected: FAIL because diagnostics currently depend on the minimal `DiagnosticsSnapshot` shape.

**Step 3: Write minimal implementation**

Introduce a bridge so assertions can consume recorder-backed snapshot data:

- either replace `DiagnosticsSnapshot` with a richer adapter over `blinc_recorder::TreeSnapshot`
- or keep `DiagnosticsSnapshot` as a compatibility wrapper backed by `TreeSnapshot`

Keep current callers working, but make the richer tree data the default path for new automation work.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app headless_asserts_can_read_text_from_tree_snapshot_backed_probe -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_assert.rs crates/blinc_app/src/headless_runner.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/tests.rs
git commit -m "feat: back headless diagnostics with recorder snapshots"
```

### Task 3: Extend the existing headless scenario DSL with action commands

**Files:**
- Modify: `crates/blinc_app/src/headless_scenario.rs`
- Modify: `crates/blinc_app/src/headless_runner.rs`
- Modify: `crates/blinc_app/src/headless_report.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn scenario_parses_click_fill_and_press_steps() {}

#[test]
fn headless_runner_returns_structured_failure_for_unhandled_action_step() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app scenario_parses_click_fill_and_press_steps -- --exact`
Expected: FAIL because `ScenarioStep` currently only supports `wait`, `tick`, and assertion steps.

**Step 3: Write minimal implementation**

Extend the existing `ScenarioStep` enum with a small MVP action set:

- `click`
- `fill`
- `press`
- `scroll`
- optional `snapshot` or `export_trace`

Keep existing JSON compatibility for the current diagnostics workflow.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app scenario_parses_click_fill_and_press_steps -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_scenario.rs crates/blinc_app/src/headless_runner.rs crates/blinc_app/src/headless_report.rs crates/blinc_app/src/tests.rs
git commit -m "feat: extend headless scenario steps for automation actions"
```

### Task 4: Finish programmatic interaction dispatch in `blinc_layout::selector`

**Files:**
- Modify: `crates/blinc_layout/src/selector/handle.rs`
- Modify: `crates/blinc_core/src/context_state.rs`
- Test: `crates/blinc_layout/src/selector/handle.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn element_handle_click_dispatches_into_runtime_event_path() {}

#[test]
fn element_handle_focus_updates_context_focus_state() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_layout element_handle_click_dispatches_into_runtime_event_path -- --nocapture`
Expected: FAIL because `ElementHandle::dispatch_event()` is still a TODO.

**Step 3: Write minimal implementation**

Complete the `ElementHandle` dispatch plumbing so it can drive the existing event system for:

- click
- focus
- blur
- key down

Do not create a second event-dispatch surface. Wire into the existing runtime/event-router path.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_layout element_handle_click_dispatches_into_runtime_event_path -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_layout/src/selector/handle.rs crates/blinc_core/src/context_state.rs
git commit -m "feat: complete selector event dispatch for automation"
```

### Task 5: Add semantic locator resolution to the existing selector registry

**Files:**
- Modify: `crates/blinc_layout/src/selector/mod.rs`
- Modify: `crates/blinc_layout/src/selector/registry.rs`
- Create: `crates/blinc_layout/src/selector/semantic.rs`
- Modify: `crates/blinc_layout/src/recorder_bridge.rs`
- Test: `crates/blinc_layout/src/selector/semantic.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn semantic_query_matches_role_and_label() {}

#[test]
fn semantic_query_reports_ambiguity_when_multiple_nodes_match() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_layout semantic_query_matches_role_and_label -- --nocapture`
Expected: FAIL because selector support is currently ID-centric.

**Step 3: Write minimal implementation**

Extend the existing selector layer with semantic query support:

- role
- text
- label
- placeholder
- within
- nth

Capture resolution evidence in a reusable structure that can later be recorded into `RecordingExport`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_layout semantic_query_matches_role_and_label -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_layout/src/selector/mod.rs crates/blinc_layout/src/selector/registry.rs crates/blinc_layout/src/selector/semantic.rs crates/blinc_layout/src/recorder_bridge.rs
git commit -m "feat: add semantic locator resolution to selector layer"
```

### Task 6: Add an app-level `AutomationSession` that reuses the current headless and windowed runtimes

**Files:**
- Create: `crates/blinc_app/src/automation_session.rs`
- Modify: `crates/blinc_app/src/lib.rs`
- Modify: `crates/blinc_app/src/windowed/mod.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn automation_session_executes_click_and_assert_commands_in_headless_mode() {}

#[test]
fn automation_session_returns_trace_linked_failure_details() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app automation_session_executes_click_and_assert_commands_in_headless_mode -- --exact`
Expected: FAIL because no shared session type exists yet.

**Step 3: Write minimal implementation**

Add an `AutomationSession` inside `blinc_app` that:

- consumes the existing scenario and selector infrastructure
- appends command and assertion trace entries into `RecordingExport`
- can target headless first, then desktop

Keep the command surface intentionally small in MVP.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app automation_session_executes_click_and_assert_commands_in_headless_mode -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/automation_session.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/windowed/mod.rs crates/blinc_app/src/tests.rs
git commit -m "feat: add app-level automation session"
```

### Task 7: Add desktop execution parity on top of the same session and selector path

**Files:**
- Modify: `crates/blinc_app/src/automation_session.rs`
- Modify: `crates/blinc_app/src/windowed/mod.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn desktop_session_uses_same_command_types_as_headless() {}

#[test]
fn desktop_and_headless_runs_produce_matching_assertion_outcomes() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app desktop_and_headless_runs_produce_matching_assertion_outcomes -- --exact`
Expected: FAIL because desktop execution parity is not wired through the shared session.

**Step 3: Write minimal implementation**

Reuse the same `AutomationSession` command types for the windowed runtime, and tag runtime mode differences explicitly in trace output rather than forking the API.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app desktop_and_headless_runs_produce_matching_assertion_outcomes -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/automation_session.rs crates/blinc_app/src/windowed/mod.rs crates/blinc_app/src/tests.rs
git commit -m "feat: add desktop parity for automation sessions"
```

### Task 8: Build state-machine playbooks on top of the existing FSM runtime

**Files:**
- Create: `crates/blinc_app/src/playbook.rs`
- Modify: `crates/blinc_app/src/lib.rs`
- Modify: `crates/blinc_core/src/fsm.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn playbook_parses_named_states_and_transitions() {}

#[test]
fn playbook_compiles_into_existing_fsm_runtime_types() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app playbook_compiles_into_existing_fsm_runtime_types -- --exact`
Expected: FAIL because playbook parsing and FSM bridging do not exist yet.

**Step 3: Write minimal implementation**

Add playbook parsing and compilation that reuses the existing FSM runtime. Keep the compiler thin:

- parse states and transitions
- validate missing states and invalid edges
- compile into existing FSM types or a small adapter around them

Do not add a second state-machine implementation.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app playbook_compiles_into_existing_fsm_runtime_types -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/playbook.rs crates/blinc_app/src/lib.rs crates/blinc_core/src/fsm.rs crates/blinc_app/src/tests.rs
git commit -m "feat: add playbook support on top of existing fsm runtime"
```

### Task 9: Add CLI commands by reusing the current headless diagnostics entry path

**Files:**
- Modify: `crates/blinc_cli/src/main.rs`
- Modify: `crates/blinc_cli/src/project.rs`
- Create: `crates/blinc_cli/src/automation.rs`
- Test: `crates/blinc_cli/src/automation.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn cli_parses_automation_run_command() {}

#[test]
fn cli_parses_playbook_validate_command() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_cli cli_parses_automation_run_command -- --nocapture`
Expected: FAIL because the automation CLI subcommands are missing.

**Step 3: Write minimal implementation**

Add commands such as:

- `blinc automation run --scenario path.json --headless`
- `blinc automation validate --playbook path.yaml`
- `blinc automation export-diagram --playbook path.yaml -o diagram.svg`

Reuse existing `run_loaded_scenario_with_probe()`-style execution paths instead of inventing a separate CLI runtime.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_cli cli_parses_automation_run_command -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/main.rs crates/blinc_cli/src/project.rs crates/blinc_cli/src/automation.rs
git commit -m "feat: add automation and playbook cli entrypoints"
```

### Task 10: Upgrade the existing debugger panels into a forensic trace workflow

**Files:**
- Modify: `crates/blinc_debugger/src/app.rs`
- Modify: `crates/blinc_debugger/src/panels/mod.rs`
- Create: `crates/blinc_debugger/src/panels/command_panel.rs`
- Create: `crates/blinc_debugger/src/panels/evidence_panel.rs`
- Modify: `crates/blinc_debugger/src/panels/tree_panel.rs`
- Modify: `crates/blinc_debugger/src/panels/inspector_panel.rs`
- Modify: `crates/blinc_debugger/src/panels/timeline_panel.rs`
- Test: `crates/blinc_debugger/src/app.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn debugger_reads_enriched_recording_export_and_shows_command_stream() {}

#[test]
fn inspector_shows_locator_resolution_and_assertion_context() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_debugger debugger_reads_enriched_recording_export_and_shows_command_stream -- --exact`
Expected: FAIL because the debugger only surfaces the current recording-oriented data.

**Step 3: Write minimal implementation**

Extend the current debugger instead of replacing it:

- keep the existing tree, timeline, and inspector foundations
- add command and evidence panels
- surface assertion failures and locator resolution metadata from the enriched `RecordingExport`

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_debugger debugger_reads_enriched_recording_export_and_shows_command_stream -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_debugger/src/app.rs crates/blinc_debugger/src/panels/mod.rs crates/blinc_debugger/src/panels/command_panel.rs crates/blinc_debugger/src/panels/evidence_panel.rs crates/blinc_debugger/src/panels/tree_panel.rs crates/blinc_debugger/src/panels/inspector_panel.rs crates/blinc_debugger/src/panels/timeline_panel.rs
git commit -m "feat: upgrade debugger for forensic trace analysis"
```

### Task 11: Add end-to-end parity fixtures, docs, and migration notes

**Files:**
- Create: `crates/blinc_app/tests/fixtures/login_scenario.json`
- Create: `crates/blinc_app/tests/fixtures/login_playbook.yaml`
- Create: `crates/blinc_app/tests/automation_parity.rs`
- Modify: `README.md`
- Modify: `crates/blinc_app/README.md`
- Modify: `crates/blinc_recorder/README.md`
- Modify: `crates/blinc_debugger/README.md`
- Modify: `docs/headless-diagnostics.md`
- Test: `crates/blinc_app/tests/automation_parity.rs`

**Step 1: Write the failing tests**

Add integration tests for:

```rust
#[test]
fn login_scenario_matches_between_headless_and_desktop() {}

#[test]
fn playbook_execution_emits_same_required_trace_fields_as_scenario_execution() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app --test automation_parity -- --nocapture`
Expected: FAIL because the parity fixtures and enriched traces are not wired yet.

**Step 3: Write minimal implementation**

Add fixtures and documentation that explain:

1. how existing headless diagnostics evolve into automation scenarios
2. how enriched `RecordingExport` acts as the shared evidence container
3. how to run headless and desktop automation
4. how to open failures in the debugger

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app --test automation_parity -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/tests/fixtures/login_scenario.json crates/blinc_app/tests/fixtures/login_playbook.yaml crates/blinc_app/tests/automation_parity.rs README.md crates/blinc_app/README.md crates/blinc_recorder/README.md crates/blinc_debugger/README.md docs/headless-diagnostics.md
git commit -m "docs: add agent debugging workflow and parity coverage"
```

