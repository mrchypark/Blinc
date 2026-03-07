# Agent Debugging Platform Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Blinc-native agent-debugging platform with a shared trace model, Rust-native automation API, scenario and state-machine playbooks, desktop and headless execution, and a forensic debugger UI.

**Architecture:** Introduce new shared crates for trace, automation, and playbooks, then refit `blinc_recorder`, `blinc_app`, and `blinc_debugger` around those types. Keep desktop and headless execution on one canonical command model so the debugger can analyze both with the same evidence pipeline.

**Tech Stack:** Rust 2021, Cargo workspace crates, `serde`, `serde_json`, `serde_yaml`, `anyhow`, `thiserror`, existing Blinc runtime crates, and the current recorder/debugger foundations.

---

## Global Constraints / Guardrails

- Follow TDD for every behavior: failing test first, confirm failure, then write the minimum implementation.
- Do not break existing recorder/debugger use cases while introducing the new trace model; provide explicit compatibility shims where needed.
- Keep the MVP desktop-first plus headless; mobile is out of scope in this plan.
- Prefer Rust-native APIs and types over generated or JS-first surfaces.
- Support both stable ID and semantic locators in the canonical model.
- Scenario DSL and state-machine playbooks must compile to the same execution plan type.

### Task 1: Create the shared trace crate and canonical envelope

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/blinc_trace/Cargo.toml`
- Create: `crates/blinc_trace/src/lib.rs`
- Create: `crates/blinc_trace/src/envelope.rs`
- Create: `crates/blinc_trace/src/streams.rs`
- Create: `crates/blinc_trace/src/artifact.rs`
- Test: `crates/blinc_trace/src/lib.rs`

**Step 1: Write the failing tests**

Add unit tests for:

```rust
#[test]
fn trace_envelope_round_trips_with_command_and_assertion_streams() {}

#[test]
fn trace_entry_keeps_sequence_and_causal_command_id() {}

#[test]
fn artifact_descriptor_requires_stable_kind_and_uri() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_trace`
Expected: FAIL because the crate does not exist yet.

**Step 3: Write minimal implementation**

Create the crate with:

- `TraceEnvelope`
- `TraceEntry`
- `TraceStreamKind`
- `TraceCommand`
- `TraceAssertion`
- `TraceArtifactDescriptor`

Use explicit schema versioning and make every trace entry serializable with `serde`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_trace`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/blinc_trace
git commit -m "feat: add shared trace envelope crate"
```

### Task 2: Create the automation crate with canonical commands and locators

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/blinc_automation/Cargo.toml`
- Create: `crates/blinc_automation/src/lib.rs`
- Create: `crates/blinc_automation/src/command.rs`
- Create: `crates/blinc_automation/src/locator.rs`
- Create: `crates/blinc_automation/src/assert.rs`
- Create: `crates/blinc_automation/src/error.rs`
- Test: `crates/blinc_automation/src/lib.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn semantic_locator_normalizes_within_and_nth_filters() {}

#[test]
fn click_command_serializes_with_locator_target() {}

#[test]
fn assertion_request_keeps_expected_text_payload() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_automation`
Expected: FAIL because the crate does not exist yet.

**Step 3: Write minimal implementation**

Define:

- `AutomationCommand`
- `Locator`
- `SemanticLocator`
- `ResolvedTarget`
- `AssertionRequest`
- `AutomationError`

Support both:

- `Locator::Id(String)`
- `Locator::Semantic(SemanticLocator)`

Include `role`, `text`, `label`, `placeholder`, `within`, and `nth` fields in the semantic model.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_automation`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/blinc_automation
git commit -m "feat: add automation command and locator crate"
```

### Task 3: Add semantic metadata extraction hooks to the layout/runtime side

**Files:**
- Modify: `crates/blinc_core/src/context_state.rs`
- Modify: `crates/blinc_layout/src/recorder_bridge.rs`
- Create: `crates/blinc_layout/src/semantic_snapshot.rs`
- Test: `crates/blinc_layout/src/semantic_snapshot.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn snapshot_extracts_role_label_and_placeholder_metadata() {}

#[test]
fn snapshot_marks_nodes_as_semantically_matchable() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_layout semantic_snapshot -- --nocapture`
Expected: FAIL because semantic snapshot extraction is missing.

**Step 3: Write minimal implementation**

Introduce a semantic metadata structure that can be attached to snapshot nodes:

- `role`
- `label`
- `placeholder`
- normalized text content
- matchability flags

Update recorder bridge conversion so captured tree snapshots carry this metadata into the trace path.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_layout semantic_snapshot -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_core/src/context_state.rs crates/blinc_layout/src/recorder_bridge.rs crates/blinc_layout/src/semantic_snapshot.rs
git commit -m "feat: capture semantic locator metadata in layout snapshots"
```

### Task 4: Upgrade `blinc_recorder` to emit trace-compatible commands, events, and snapshots

**Files:**
- Modify: `crates/blinc_recorder/Cargo.toml`
- Modify: `crates/blinc_recorder/src/lib.rs`
- Modify: `crates/blinc_recorder/src/session/recording.rs`
- Modify: `crates/blinc_recorder/src/capture/tree.rs`
- Create: `crates/blinc_recorder/src/trace_bridge.rs`
- Test: `crates/blinc_recorder/src/session/recording.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn export_contains_trace_entries_for_runtime_events_and_state_snapshots() {}

#[test]
fn recorder_assigns_monotonic_sequence_numbers_to_trace_entries() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_recorder export_contains_trace_entries_for_runtime_events_and_state_snapshots -- --nocapture`
Expected: FAIL because trace-backed export does not exist.

**Step 3: Write minimal implementation**

Add a trace bridge that:

- maps recorded runtime events to `blinc_trace` entries
- maps state snapshots to `blinc_trace` entries
- preserves sequence numbers and timestamps
- leaves the old export fields intact for compatibility during the migration

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_recorder export_contains_trace_entries_for_runtime_events_and_state_snapshots -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_recorder/Cargo.toml crates/blinc_recorder/src/lib.rs crates/blinc_recorder/src/session/recording.rs crates/blinc_recorder/src/capture/tree.rs crates/blinc_recorder/src/trace_bridge.rs
git commit -m "feat: bridge recorder output into canonical trace entries"
```

### Task 5: Add headless automation runtime contract to `blinc_app`

**Files:**
- Create: `crates/blinc_app/src/headless_runtime.rs`
- Create: `crates/blinc_app/src/headless_runner.rs`
- Modify: `crates/blinc_app/src/lib.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn headless_runner_executes_click_command_and_records_trace() {}

#[test]
fn headless_runner_returns_assertion_failure_with_trace_reference() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app headless_runner_executes_click_command_and_records_trace -- --exact`
Expected: FAIL because the headless automation runner is missing.

**Step 3: Write minimal implementation**

Create a headless runtime that:

- accepts canonical automation commands
- advances frames without creating a native window
- invokes recorder/trace capture
- returns assertion outcomes with trace linkage

Start with a narrow action set:

- click
- fill
- wait
- assert exists
- assert text contains

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app headless_runner_executes_click_command_and_records_trace -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/headless_runtime.rs crates/blinc_app/src/headless_runner.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/tests.rs
git commit -m "feat: add headless automation runtime"
```

### Task 6: Add desktop automation session execution to `blinc_app`

**Files:**
- Create: `crates/blinc_app/src/automation_session.rs`
- Modify: `crates/blinc_app/src/lib.rs`
- Modify: `crates/blinc_app/src/windowed/mod.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn desktop_session_executes_command_against_windowed_app() {}

#[test]
fn desktop_and_headless_sessions_produce_matching_assertion_outcomes() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app desktop_and_headless_sessions_produce_matching_assertion_outcomes -- --exact`
Expected: FAIL because there is no shared desktop automation session path.

**Step 3: Write minimal implementation**

Add:

- `AutomationSession`
- desktop session start helper
- command dispatch into the windowed runtime
- trace output aligned with the headless runner

Keep command execution synchronous first if that simplifies the MVP.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app desktop_and_headless_sessions_produce_matching_assertion_outcomes -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/src/automation_session.rs crates/blinc_app/src/lib.rs crates/blinc_app/src/windowed/mod.rs crates/blinc_app/src/tests.rs
git commit -m "feat: add desktop automation session runtime"
```

### Task 7: Create the playbook crate and compile both DSL forms into one execution plan

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/blinc_playbook/Cargo.toml`
- Create: `crates/blinc_playbook/src/lib.rs`
- Create: `crates/blinc_playbook/src/scenario.rs`
- Create: `crates/blinc_playbook/src/state_machine.rs`
- Create: `crates/blinc_playbook/src/plan.rs`
- Create: `crates/blinc_playbook/src/validate.rs`
- Test: `crates/blinc_playbook/src/lib.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn scenario_document_compiles_into_execution_plan() {}

#[test]
fn state_machine_playbook_compiles_into_execution_plan() {}

#[test]
fn invalid_transition_is_reported_during_validation() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_playbook`
Expected: FAIL because the crate does not exist yet.

**Step 3: Write minimal implementation**

Implement:

- a scenario parser
- a state-machine parser
- a shared `ExecutionPlan`
- validator support for missing states, invalid transitions, and duplicate step IDs

Use `serde_yaml` and `serde_json` for input formats if that keeps the MVP simple.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_playbook`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/blinc_playbook
git commit -m "feat: add scenario and state-machine playbook compiler"
```

### Task 8: Add CLI entry points for scenario runs and playbook validation

**Files:**
- Modify: `crates/blinc_cli/Cargo.toml`
- Modify: `crates/blinc_cli/src/main.rs`
- Create: `crates/blinc_cli/src/automation.rs`
- Test: `crates/blinc_cli/src/automation.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn cli_parses_run_scenario_command() {}

#[test]
fn cli_parses_validate_playbook_command() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_cli cli_parses_run_scenario_command -- --nocapture`
Expected: FAIL because automation commands are missing from the CLI.

**Step 3: Write minimal implementation**

Add subcommands such as:

- `blinc automation run --scenario path.yaml --headless`
- `blinc automation validate --playbook path.yaml`
- `blinc automation export-diagram --playbook path.yaml -o diagram.svg`

Wire command handlers to the new playbook and automation crates.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_cli cli_parses_run_scenario_command -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/Cargo.toml crates/blinc_cli/src/main.rs crates/blinc_cli/src/automation.rs
git commit -m "feat: add automation and playbook CLI commands"
```

### Task 9: Add locator resolution tracing and interactability diagnostics

**Files:**
- Modify: `crates/blinc_automation/src/locator.rs`
- Modify: `crates/blinc_app/src/automation_session.rs`
- Modify: `crates/blinc_trace/src/streams.rs`
- Test: `crates/blinc_app/src/tests.rs`

**Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn ambiguous_semantic_locator_produces_locator_resolution_trace_entry() {}

#[test]
fn non_interactable_target_returns_machine_readable_error_code() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app ambiguous_semantic_locator_produces_locator_resolution_trace_entry -- --exact`
Expected: FAIL because locator resolution evidence is not emitted.

**Step 3: Write minimal implementation**

Record:

- the original locator query
- the candidate set
- the chosen match if any
- a stable failure code for ambiguity or interactability failures

Store all of that in the canonical trace.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app ambiguous_semantic_locator_produces_locator_resolution_trace_entry -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_automation/src/locator.rs crates/blinc_app/src/automation_session.rs crates/blinc_trace/src/streams.rs crates/blinc_app/src/tests.rs
git commit -m "feat: add locator resolution evidence to automation traces"
```

### Task 10: Upgrade `blinc_debugger` into a forensic trace debugger

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
fn debugger_loads_trace_envelope_and_exposes_command_stream() {}

#[test]
fn inspector_shows_locator_resolution_evidence_for_selected_node() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_debugger debugger_loads_trace_envelope_and_exposes_command_stream -- --exact`
Expected: FAIL because the debugger still assumes the old recording-only model.

**Step 3: Write minimal implementation**

Update the debugger to:

- load `blinc_trace` envelopes
- show command entries alongside runtime events
- add an evidence panel
- surface locator resolution and assertion failure metadata in the inspector

Do not attempt polished visual design before the data path is proven.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_debugger debugger_loads_trace_envelope_and_exposes_command_stream -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_debugger/src/app.rs crates/blinc_debugger/src/panels/mod.rs crates/blinc_debugger/src/panels/command_panel.rs crates/blinc_debugger/src/panels/evidence_panel.rs crates/blinc_debugger/src/panels/tree_panel.rs crates/blinc_debugger/src/panels/inspector_panel.rs crates/blinc_debugger/src/panels/timeline_panel.rs
git commit -m "feat: turn debugger into a trace forensic tool"
```

### Task 11: Add example fixtures and parity tests for desktop and headless runs

**Files:**
- Create: `crates/blinc_app/tests/fixtures/login_scenario.yaml`
- Create: `crates/blinc_app/tests/fixtures/login_playbook.yaml`
- Create: `crates/blinc_app/tests/automation_parity.rs`
- Test: `crates/blinc_app/tests/automation_parity.rs`

**Step 1: Write the failing tests**

Add integration tests for:

```rust
#[test]
fn login_scenario_matches_between_desktop_and_headless() {}

#[test]
fn login_playbook_validation_and_execution_share_same_trace_schema() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_app --test automation_parity -- --nocapture`
Expected: FAIL because fixtures and parity harness are missing.

**Step 3: Write minimal implementation**

Add representative fixtures and wire them through both runtimes. Assert:

- equal assertion outcomes
- same top-level trace schema version
- same required stream kinds

Allow mode-specific artifact differences only when explicitly tagged.

**Step 4: Run test to verify it passes**

Run: `cargo test -p blinc_app --test automation_parity -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_app/tests/fixtures/login_scenario.yaml crates/blinc_app/tests/fixtures/login_playbook.yaml crates/blinc_app/tests/automation_parity.rs
git commit -m "test: add desktop and headless automation parity coverage"
```

### Task 12: Document the new developer workflow and migration path

**Files:**
- Modify: `README.md`
- Modify: `crates/blinc_recorder/README.md`
- Modify: `crates/blinc_debugger/README.md`
- Create: `docs/book/src/tooling/agent-debugging.md`
- Test: manual doc sanity check

**Step 1: Write the missing-doc checklist**

List the doc gaps to close:

- how to launch automation sessions
- how to run headless scenarios
- how to author playbooks
- how to open traces in the debugger
- how old recording exports map to the new trace model

**Step 2: Verify the current docs are insufficient**

Run: `rg -n "automation|playbook|headless|trace forensic" README.md crates/blinc_recorder/README.md crates/blinc_debugger/README.md docs/book/src`
Expected: existing docs do not cover the new workflow end-to-end.

**Step 3: Write minimal implementation**

Update the docs with one coherent flow:

1. launch or run a scenario
2. collect a trace
3. validate a playbook
4. inspect the trace in the debugger

Document compatibility expectations for existing recorder/debugger users.

**Step 4: Sanity check the docs**

Run: `rg -n "agent debugging|playbook|trace" README.md crates/blinc_recorder/README.md crates/blinc_debugger/README.md docs/book/src/tooling/agent-debugging.md`
Expected: the new workflow is discoverable from both root and crate docs.

**Step 5: Commit**

```bash
git add README.md crates/blinc_recorder/README.md crates/blinc_debugger/README.md docs/book/src/tooling/agent-debugging.md
git commit -m "docs: add agent debugging platform workflow"
```

