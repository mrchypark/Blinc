# Agent Debugging Platform Design (Blinc)

## Problem

Blinc already has three important pieces of a developer-tooling stack:

- `blinc_recorder` captures runtime events, tree snapshots, replay state, and debug-server exports
- `blinc_debugger` loads those exports and provides a basic tree/preview/inspector/timeline UI
- `blinc_app` already includes app-level headless diagnostics primitives such as `HeadlessRuntime`, `HeadlessScenario`, `headless_runner`, and `HeadlessReport`

The gap is not a missing foundation. The gap is that these pieces are still separate and too limited for agent-driven debugging. An agent cannot yet use one coherent Rust-native API to drive a Blinc app, inject actions, resolve elements by both stable IDs and semantic locators, capture causally linked trace evidence, and inspect the failure later in a debugger that understands commands, assertions, and locator resolution.

The most important design rule for this work is:

- do not rebuild capabilities that the repository already has

This platform must be an evolution of the current recorder, debugger, selector, and headless-diagnostics work, not a parallel product layered beside them.

## Scope

- Desktop application automation support
- Headless execution support with the same trace schema as desktop
- Rust-native automation API for agent-driven execution
- Straight-line scenario DSL support by extending the existing headless scenario model
- State-machine playbook support by building on the existing FSM runtime
- Unified trace capture of commands, locator resolution, runtime events, state snapshots, render evidence, and assertions
- Debugger upgrades for forensic analysis of failed runs

## Explicit Non-Goals

- Mobile automation as an MVP requirement
- Full Playwright API compatibility
- Replacing existing recorder/debugger/headless modules with a parallel stack
- Making `probar` a runtime dependency
- JavaScript-first automation surfaces in v1

## Current Assets To Reuse

### `blinc_recorder`

Already provides:

- event capture
- tree snapshot capture
- replay player
- debug-server export transport
- `RecordingExport` as the current serialization boundary

### `blinc_debugger`

Already provides:

- file and server import
- existing tree, inspector, preview, and timeline panels
- replay-driven cursor and snapshot navigation

### `blinc_app`

Already provides:

- `HeadlessRuntime`
- `HeadlessScenario`
- `ScenarioStep`
- `run_loaded_scenario_with_probe`
- `HeadlessReport`

### `blinc_layout`

Already provides:

- stable ID query APIs in `selector`
- `ElementRegistry`
- `ElementHandle`
- partial programmatic control surfaces
- recorder bridge hooks for events and tree snapshots

### `blinc_core`

Already provides:

- FSM runtime primitives in `fsm.rs`

## External Reference: `probar`

The closest external reference is [`paiml/probar`](https://github.com/paiml/probar).

Useful ideas to borrow:

- library plus CLI split
- headless-first testing ergonomics
- deterministic replay mindset
- scenario and state-machine style inputs
- validation and diagram export as first-class tooling

Reasons not to embed it directly:

- `probar` is designed around WASM/TUI/CDP targets, not Blinc's internal render tree and context state model
- Blinc can observe far richer internal state than a generic automation backend
- this platform needs Blinc-native tree snapshots, locator evidence, and debugger drill-down

Decision:

- use `probar` as a product reference
- keep implementation Blinc-native
- prioritize compatibility by concept, not by dependency

## Options Considered

### Option A: Add a separate automation stack with new foundational crates

- create new automation, trace, and playbook crates beside current recorder/debugger code

**Pros**

- clean conceptual layering on paper
- easy to describe in architecture diagrams

**Cons**

- duplicates existing headless diagnostics, trace export, selector, and debugger work
- introduces migration and consistency problems immediately
- violates the primary rule to avoid rebuilding what already exists

### Option B: Evolve the existing recorder, debugger, app, selector, and FSM layers (Chosen)

- keep current crates
- extend them in-place around one shared command-and-trace model
- extract crates later only if the APIs prove stable and reusable

**Pros**

- best reuse of tested code already in the repository
- lowest rework risk
- preserves existing workflows while growing them toward agent-grade debugging
- keeps the debugger and recorder aligned with the runtime internals they already understand

**Cons**

- requires more discipline to avoid overloading existing crates
- needs explicit boundaries to prevent feature sprawl

### Option C: Make `probar` the primary engine and attach Blinc as a backend

**Pros**

- strong external reference
- some concepts already exist

**Cons**

- gives up Blinc-native control over the trace and state models
- creates dependency and integration risk for the most critical developer tooling
- forces Blinc to flatten internal state into a foreign abstraction too early

## Decision

Choose **Option B**.

The MVP should extend the existing architecture first. No new foundational crate should be introduced in MVP unless reuse is proven impossible after targeted implementation work.

## Architectural Boundaries

### 1. `blinc_recorder` becomes the canonical trace home

Do not create a parallel trace package in v1.

Instead:

- evolve `RecordingExport`
- add explicit trace entry types inside `blinc_recorder`
- preserve compatibility for existing event/snapshot consumers during migration

New trace responsibilities inside `blinc_recorder`:

- command stream
- locator resolution stream
- assertion stream
- artifact descriptors
- causal links between command, runtime event, and failure evidence

The current event and snapshot capture pipeline stays in `blinc_recorder`; it is extended rather than replaced.

### 2. `blinc_app` owns automation sessions and headless execution

Do not create a new automation foundation crate in MVP.

Instead:

- extend the existing `headless_*` modules
- add an `AutomationSession` inside `blinc_app`
- reuse the current headless runner and runtime instead of recreating them

`blinc_app` responsibilities:

- launching or attaching to an app runtime
- running commands in headless and desktop modes
- coordinating command execution and assertion evaluation
- returning trace-linked outcomes

### 3. `blinc_layout::selector` becomes the locator and interaction engine

Do not create a parallel locator subsystem.

Instead:

- keep stable ID lookup on top of the existing selector registry
- add semantic locator resolution to the selector layer
- finish the currently incomplete event-dispatch path in `ElementHandle`

Selector responsibilities:

- stable ID lookup
- semantic lookup
- interactability checks
- locator ambiguity reporting
- dispatch support for click/focus/input-like operations

### 4. `blinc_core::fsm` remains the state-machine substrate

Do not create a separate state-machine foundation for playbooks.

Instead:

- compile playbook transitions into wrappers or adapters around the current FSM runtime
- keep playbook parsing and validation close to `blinc_app` and `blinc_cli`

This avoids maintaining two state-machine systems in one repository.

### 5. `blinc_cli` provides user-facing automation and playbook entry points

Responsibilities:

- run scenarios
- validate playbooks
- export diagrams
- emit traces and reports

The CLI should reuse the existing headless diagnostics path where possible instead of inventing a separate launch model.

### 6. `blinc_debugger` evolves into a forensic trace debugger

Do not replace the existing debugger UI.

Instead:

- extend current panels
- add only the missing panels and data sources
- keep the existing replay-oriented controls where they still help

New capabilities:

- command log
- assertion failure surfacing
- locator resolution evidence
- richer state tree inspection
- artifact drill-down

## Runtime Model

The runtime model should be command-driven and trace-backed.

Flow:

1. Rust API, scenario, or playbook emits a canonical command
2. selector resolves a target by stable ID or semantic query
3. runtime checks interactability and executes the action in desktop or headless mode
4. recorder appends command, locator evidence, runtime events, and resulting snapshots into the trace
5. assertions append pass/fail outcomes into the same trace
6. debugger reconstructs the timeline from this shared evidence

This is intentionally stronger than simple playback of a recording file. The system is built around causality, not only capture.

## Locator Strategy

The MVP must support both:

- stable IDs
- semantic locators

Supported semantic families in MVP:

- `role`
- `text`
- `label`
- `placeholder`
- `within`
- `nth`

Execution rule:

- all locator forms normalize to one resolved-target model inside the selector layer
- ambiguity is an explicit runtime error and an explicit trace event
- the trace stores the original query, candidate set, and final match if one exists

The existing selector registry is the starting point. Semantic support is layered on top of it rather than rebuilt elsewhere.

## Trace Model

The trace should evolve from `RecordingExport`, not live beside it.

Each entry must include:

- sequence number
- monotonic timestamp
- frame index when relevant
- runtime mode: desktop or headless
- causal link to the originating command when relevant

Trace streams to add:

### Command

- requested operation
- locator or direct target
- input payload
- timeout and budget metadata

### Runtime Event

- mouse, key, focus, hover, scroll, and custom events

### Locator Resolution

- original query
- candidates
- chosen target
- failure reason when unresolved

### State Snapshot

- element tree
- bounds
- visibility
- text
- focus and hover state
- semantic metadata
- event-handler metadata where available

### Render Evidence

- framebuffer image or lightweight render summary
- optional diff links

### Assertion

- assertion request
- pass or fail
- actual and expected payload
- linked evidence

### Artifact

- screenshot
- diff image
- tree dump
- exported diagram
- logs

## Headless Support

Headless support is already present in prototype form and must be extended, not recreated.

Rules:

- evolve `HeadlessScenario` instead of replacing it
- evolve `HeadlessReport` instead of replacing it
- move from `DiagnosticsSnapshot` toward richer tree-backed diagnostics by bridging to recorder snapshots
- keep desktop and headless on one shared command vocabulary and one trace schema

Headless support is required for:

- fast local iteration
- CI
- agent-driven debugging without a visible window

## Debugger UX

The debugger must shift from a simple recording viewer to a forensic workflow, but by extending existing panels where possible.

### Timeline

- keep current playback controls
- add commands, assertions, and trace markers to the same axis

### Tree Panel

- keep the current tree foundation
- add semantic metadata and diff awareness

### Inspector

- extend existing inspector data rather than replace it
- show locator match evidence, role, label, interactability, and assertion context

### Command Panel

- new panel for canonical commands and outcomes

### Evidence Panel

- new panel for screenshots, diffs, tree dumps, and logs

## Developer Experience Surfaces

The MVP should expose three user-facing entry points:

### Rust-native API

Example direction:

```rust
let app = BlincApp::launch(config)?;
let mut session = app.session();

session.locator("login.submit").click()?;
session.locator_by_role("textbox")
    .with_label("Email")
    .fill("person@example.com")?;
session.expect_text_contains("status", "Signed in")?;
```

### Scenario DSL

This should be an evolution of the existing `HeadlessScenario` format.

Current diagnostics steps already cover:

- wait
- tick
- assert exists
- assert text contains

MVP extends this with:

- click
- fill
- press
- scroll
- snapshot or export trace

### State-machine Playbook

Playbooks should support:

- named states
- transitions
- guards
- per-state assertions
- validation
- diagram export

The runtime substrate should reuse the existing FSM implementation.

## Error Model

Errors must be categorized and traceable.

Representative codes:

- `locator_not_found`
- `locator_ambiguous`
- `target_not_interactable`
- `assertion_failed`
- `trace_capture_incomplete`
- `headless_runtime_mismatch`
- `unsupported_semantic_query`

Every error must be stored in the trace with:

- stable machine code
- human-readable explanation
- linked command
- linked evidence

## Testing Strategy

Use four layers:

### 1. Unit tests

- selector and semantic locator resolution
- trace serialization within `RecordingExport`
- scenario parsing
- playbook validation
- assertion classification

### 2. Integration tests

- execute commands against representative desktop examples
- execute the same commands in headless mode
- verify equivalent assertion outcomes

### 3. Replay and golden tests

- deterministic replay with fixed seeds
- stable trace shape for known examples

### 4. Debugger smoke tests

- load a known enriched recording export
- verify timeline, tree, inspector, command, and evidence views render expected content

## Rollout Plan

### Phase 1

- extend `RecordingExport` into a richer trace
- bridge headless diagnostics onto recorder snapshots
- add action steps to the existing scenario runner

### Phase 2

- add selector-based semantic locators and interaction dispatch
- add desktop automation sessions
- add state-machine playbook validation and execution

### Phase 3

- upgrade the debugger to full forensic analysis
- add diagram export, coverage metrics, and mutation-oriented validation where justified

## Success Criteria

The MVP is successful when all of the following are true:

- an agent can drive a Blinc app through a Rust-native API without relying on a separate automation foundation
- the same scenario can run in desktop and headless modes
- stable ID and semantic locators both work
- straight-line scenarios and state-machine playbooks are both supported
- `RecordingExport` carries commands, locator evidence, runtime events, snapshots, and assertions in one trace-like export
- a failed run can be opened in the debugger and investigated without rerunning the app

