# Agent Debugging Platform Design (Blinc)

## Problem

Blinc already has the beginnings of a recorder and debugger, but they are still oriented around post-hoc playback of a recording file. They do not yet provide a complete developer toolchain that an agent can drive directly while building or debugging a Blinc application.

The desired product is stronger than a visual viewer:

- an agent must be able to launch a Blinc app in desktop and headless modes
- inject actions through a stable Rust-native automation API
- address elements through both stable IDs and semantic locators
- persist commands, runtime events, locator resolution, state trees, render evidence, and assertions in one trace
- inspect failures in a debugger that is built around forensic trace analysis rather than simple recording playback

The current `blinc_recorder` and `blinc_debugger` crates are useful foundations, but they do not yet define a unified automation runtime, a canonical trace model, or a debugger UX designed for agent-generated failures.

## Scope

- Desktop application automation support
- Headless execution support with the same trace format as desktop
- Rust-native automation API for agent-driven execution
- Scenario DSL and state-machine playbook support
- Unified trace model covering commands, runtime events, state snapshots, render evidence, and assertions
- Debugger UI upgrades for command/event/trace forensics

## Explicit Non-Goals

- Mobile automation as an MVP requirement
- Full Playwright API compatibility
- A dependency on `probar` as the core runtime implementation
- Remote browser/CDP automation as the primary execution target
- Production end-user telemetry features

## External Reference: `probar`

The closest external reference is [`paiml/probar`](https://github.com/paiml/probar).

Relevant qualities to borrow:

- separate library and CLI surfaces
- familiar automation ergonomics
- deterministic replay and trace-oriented execution
- playbook validation and state-machine support
- headless-first execution story

Reasons not to embed it directly:

- `probar` is designed around WASM/TUI/CDP-style targets, not Blinc's internal render tree and context state model
- Blinc already has privileged access to element state, focus, hover, layout, and render metadata that would be awkward to flatten into a foreign runtime
- the debugger needs Blinc-native state tree and locator evidence, not only generic automation output

Decision:

- use `probar` as a product and architecture reference
- build a Blinc-native runtime, trace model, and debugger
- keep future compatibility possible through import/export or a backend adapter, but do not make that a v1 dependency

## Options Considered

### Option A: Extend the existing recorder/debugger incrementally

- Keep all work inside `blinc_recorder` and `blinc_debugger`
- Add automation hooks in-place

**Pros**

- Lowest initial package churn
- Reuses existing recording model quickly

**Cons**

- Pushes unrelated concerns into crates that already have mixed responsibilities
- Makes it harder to separate automation commands from trace data
- Risks another rewrite once scenario/playbook/headless features expand

### Option B: Build a Blinc-native automation platform with `probar`-shaped UX (Chosen)

- Create new automation and playbook layers
- Keep recorder/debugger, but reshape them around the new trace model
- Expose a Rust-native automation API plus CLI/playbook surfaces

**Pros**

- Best fit for Blinc internals
- Supports both desktop and headless with one command/trace model
- Lets the debugger evolve into a proper forensic tool
- Keeps room for future `probar`-style compatibility without architectural lock-in

**Cons**

- Requires more initial design work
- Introduces new crates and migration work

### Option C: Make `probar` the primary engine and add a Blinc backend

- Treat Blinc as a target adapter
- Reuse as much of `probar` as possible

**Pros**

- Strong external reference
- Some concepts already exist

**Cons**

- Gives up Blinc-native control over trace and state semantics
- Makes core debugger features depend on a third-party runtime model
- Increases integration risk for headless and render-tree-specific behavior

## Decision

Choose **Option B**.

Blinc should grow a dedicated agent-debugging platform inside the repository, using `probar` as a product reference rather than as the runtime dependency. The core of the platform should be a shared command/trace model that is reused by desktop automation, headless execution, playbooks, and the debugger.

## Product Boundary

The MVP should support:

- desktop apps
- headless runs
- Rust-native API
- scenario DSL
- state-machine playbooks
- dual locator system: stable ID and semantic locator

The MVP should not require:

- mobile automation
- JavaScript-first bindings
- browser/CDP transport as the main runtime

## Architectural Boundaries

### 1. `blinc_automation`: command and session layer

Add a new crate:

- `crates/blinc_automation`

Responsibilities:

- agent-facing Rust-native automation API
- canonical command types
- locator model
- session lifecycle
- assertion requests and results
- bridge points for desktop and headless runners

Representative types:

- `AutomationSession`
- `AutomationCommand`
- `Locator`
- `ResolvedTarget`
- `AutomationError`
- `AssertionRequest`
- `AssertionOutcome`

This crate should not own rendering, native windowing, or debugger UI.

### 2. `blinc_playbook`: scenario and state-machine compiler layer

Add a new crate:

- `crates/blinc_playbook`

Responsibilities:

- parse scenario DSL
- parse state-machine playbooks
- validate transitions and assertions
- compile both sources into one canonical execution graph
- export diagrams for state-machine playbooks

Representative types:

- `ScenarioDocument`
- `StateMachinePlaybook`
- `ExecutionPlan`
- `PlanStep`
- `PlanValidationError`

### 3. `blinc_trace`: canonical trace schema

Add a new crate:

- `crates/blinc_trace`

Responsibilities:

- shared trace envelope schema
- serialization and versioning
- trace event categories
- artifact manifesting
- evidence linking

Representative streams:

- `command`
- `runtime_event`
- `locator_resolution`
- `state_snapshot`
- `render_snapshot`
- `assertion`
- `artifact`

`blinc_recorder` should evolve to produce `blinc_trace`-compatible output rather than stay a standalone format forever.

### 4. `blinc_recorder`: capture and replay substrate

Keep:

- event capture
- tree snapshot capture
- replay clock
- debug server transport

Extend:

- trace envelope support
- locator resolution recording
- richer state snapshot metadata
- assertion and artifact hooks

The recorder becomes the low-level capture engine, not the entire product surface.

### 5. `blinc_app`: runtime execution surfaces

Add app-level execution paths:

- `desktop automation runtime`
- `headless automation runtime`

Responsibilities:

- launch app under automation
- execute canonical commands
- expose runtime hooks for state snapshots and render evidence
- guarantee trace parity between desktop and headless where possible

### 6. `blinc_debugger`: forensic debugger UI

Reposition the debugger as a trace investigation tool.

Primary panels:

- timeline
- command log
- state tree
- node inspector
- evidence panel

This debugger must be able to answer:

- which command ran
- which locator resolved, and why
- what the app state tree looked like at that moment
- what assertion failed
- what render evidence was captured

## Runtime Model

The runtime model should be command-driven, closer to CDP than to callback-heavy UI scripting.

Flow:

1. API or playbook emits an `AutomationCommand`
2. runtime resolves locator and interactability
3. runtime executes against desktop or headless app
4. runtime records the command, locator evidence, events, and resulting snapshots into the trace
5. assertions produce first-class `AssertionOutcome` entries
6. debugger reads the trace and reconstructs the investigation timeline

This is intentionally different from "playback a recording file." The system is built around causality and evidence.

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

- all user-facing locator forms normalize to one `ResolvedTarget` model
- locator ambiguity is an explicit runtime error and a trace event
- the trace stores both the original locator query and the chosen candidate

This is required for agent-grade debuggability. "Element not found" is insufficient; the system must explain whether it found zero matches, too many matches, or a non-interactable target.

## Trace Model

The trace must become the shared source of truth across all tools.

Each entry must include:

- sequence number
- monotonic timestamp
- frame index when relevant
- runtime mode: desktop or headless
- causal link to the command that triggered it when relevant

Trace streams:

### Command

- requested operation
- locator or direct target
- input payload
- timeout/budget metadata

### Runtime Event

- mouse, key, focus, hover, scroll, custom event
- raw and normalized forms when useful

### Locator Resolution

- locator query
- candidate set
- final match
- resolution rationale

### State Snapshot

- element tree
- bounds
- visibility
- text
- focus and hover state
- semantic metadata
- event-handler metadata where available

### Render Snapshot

- framebuffer image or lightweight render summary
- optional diff link

### Assertion

- request
- pass/fail
- actual and expected payload
- attached evidence

### Artifact

- screenshot
- diff image
- tree dump
- exported diagram
- debug logs

## Headless Support

Headless support is an MVP requirement.

Rules:

- headless and desktop must share the same command API
- headless and desktop must emit the same trace schema
- assertion behavior should match across both modes as closely as possible
- differences that cannot be normalized must be tagged explicitly in the trace

Headless support is not optional convenience. It is required for fast agent iteration and CI automation.

## Debugger UX

The debugger must shift from a simple playback viewer to a forensic workflow.

### Timeline

- show commands, runtime events, assertions, and snapshot capture points together
- allow filtering by stream type and failure status

### Command Log

- show canonical commands in order
- include locator and execution outcome

### State Tree

- show the current tree for a chosen point in time
- show diffs between adjacent snapshots
- support ID and semantic metadata visibility

### Inspector

- node bounds
- role/label/text
- locator match evidence
- visual props
- focus/hover/interactable state

### Evidence Panel

- assertion details
- screenshots or diffs
- exported artifacts
- trace-linked logs

## Developer Experience Surfaces

The MVP should expose three user-facing entry points:

### Rust-native API

Example direction:

```rust
let app = BlincApp::launch(config)?;
let mut session = app.session();

session.locator(Locator::id("login.submit")).click()?;
session.locator(Locator::role("textbox").with_label("Email"))
    .fill("person@example.com")?;
session.expect(Expect::text_contains(
    Locator::id("status"),
    "Signed in",
))?;
```

### Scenario DSL

Straight-line execution:

- click
- fill
- press
- scroll
- wait
- snapshot
- assert

### State-machine Playbook

Support:

- named states
- transitions
- guards
- per-state assertions
- validation
- diagram export

Both inputs must compile into the same execution plan type.

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

- locator parsing
- semantic metadata extraction
- trace envelope serialization
- playbook validation
- command normalization

### 2. Integration tests

- execute commands against representative desktop examples
- execute the same plans in headless mode
- verify equivalent assertion outcomes

### 3. Golden and replay tests

- deterministic replay with fixed seeds
- stable trace shape for known examples

### 4. Debugger smoke tests

- load a known trace
- verify timeline, command log, tree, and evidence panels render expected content

## Rollout Plan

### Phase 1

- trace envelope
- Rust-native automation session
- headless runtime
- scenario DSL

### Phase 2

- semantic locators
- state-machine playbooks
- command-aware debugger UI

### Phase 3

- diagram export
- GUI coverage metrics
- mutation and fuzzing hooks inspired by `probar`

## Success Criteria

The MVP is successful when all of the following are true:

- an agent can launch a Blinc app and drive it through a Rust-native API
- the same scenario can run in desktop and headless modes
- both ID and semantic locators are supported
- both scenario DSL and state-machine playbooks are supported
- every action produces a structured trace with commands, locator evidence, events, snapshots, and assertions
- a failed run can be opened in the debugger and investigated without rerunning the app

