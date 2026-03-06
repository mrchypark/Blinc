# Desktop Accessibility And IME Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add composition-aware text input and accessibility semantics groundwork so desktop Blinc apps can support multilingual text entry now and unlock a later full assistive-technology bridge.

**Architecture:** Add shared IME and accessibility contracts in `blinc_platform`, surface IME events through desktop event conversion, and teach `blinc_layout` widgets to consume composition events and export semantic metadata. This plan stops at a desktop accessibility backend boundary and semantic snapshot path; full OS assistive-technology adapters remain a follow-up once the semantics contract is stable.

**Tech Stack:** Rust 2021, `winit`, crate tests, layout widget tests.

---

### Task 1: Add Red Tests For IME And Accessibility Contracts

**Files:**
- Create: `crates/blinc_platform/tests/accessibility_api.rs`
- Modify: `crates/blinc_platform/src/input.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Write the failing tests**

- Add a test for composition lifecycle events: `CompositionStarted`, `CompositionUpdated`, `CompositionCommitted`, `CompositionCancelled`.
- Add a test for accessibility node roles, names, descriptions, bounds, and focusability.
- Add a test for focus traversal intents being represented without desktop-only types leaking into the shared layer.

**Step 2: Run targeted tests to verify failure**

Run:
- `cargo test -p blinc_platform --test accessibility_api -- --nocapture`

Expected: compile failures because neither IME nor accessibility modules exist.

**Step 3: Commit the red tests**

```bash
git add crates/blinc_platform/tests/accessibility_api.rs crates/blinc_platform/src/input.rs crates/blinc_platform/src/lib.rs
git commit -m "test: add desktop ime and accessibility contracts"
```

### Task 2: Add Shared IME And Accessibility Primitives

**Files:**
- Create: `crates/blinc_platform/src/accessibility.rs`
- Modify: `crates/blinc_platform/src/input.rs`
- Modify: `crates/blinc_platform/src/event.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Implement shared IME value types**

- Add composition event payloads carrying selection range and preview text.
- Keep committed text delivery separate from raw key events.

**Step 2: Implement accessibility value types**

- Add `AccessibilityNode`, `AccessibilityRole`, `AccessibilityAction`, and `AccessibilityTreeSnapshot`.
- Add an event channel for accessibility actions invoked by the platform.

**Step 3: Re-run shared API tests**

Run:
- `cargo test -p blinc_platform --test accessibility_api -- --nocapture`

Expected: PASS.

**Step 4: Commit the shared contracts**

```bash
git add crates/blinc_platform/src/accessibility.rs crates/blinc_platform/src/input.rs crates/blinc_platform/src/event.rs crates/blinc_platform/src/lib.rs crates/blinc_platform/tests/accessibility_api.rs
git commit -m "feat: add shared ime and accessibility primitives"
```

### Task 3: Implement Desktop IME Handling And Accessibility Backend Boundary

**Files:**
- Create: `extensions/blinc_platform_desktop/src/accessibility.rs`
- Modify: `extensions/blinc_platform_desktop/src/input.rs`
- Modify: `extensions/blinc_platform_desktop/src/event_loop.rs`
- Modify: `extensions/blinc_platform_desktop/src/lib.rs`
- Create: `extensions/blinc_platform_desktop/tests/ime_runtime.rs`
- Modify: `extensions/blinc_platform_desktop/tests/support.rs`

**Step 1: Write the failing desktop IME tests**

- Add a test for translating IME preedit/commit notifications into the shared composition events.
- Add a test ensuring plain keypress handling still works when composition is inactive.

**Step 2: Run targeted backend tests to verify failure**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test ime_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test ime_runtime -- --nocapture`

Expected: failures because the desktop backend only emits keyboard/mouse/touch/scroll/pinch events today.

**Step 3: Implement the minimal backend boundary**

- Translate `winit` IME callbacks into the shared input model.
- Add an accessibility backend boundary that can ingest semantic snapshots, but keep real OS bridge work explicitly out of scope for this plan.

**Step 4: Re-run backend tests**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test ime_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test ime_runtime -- --nocapture`

Expected: PASS.

**Step 5: Commit the desktop IME implementation**

```bash
git add extensions/blinc_platform_desktop/src/accessibility.rs extensions/blinc_platform_desktop/src/input.rs extensions/blinc_platform_desktop/src/event_loop.rs extensions/blinc_platform_desktop/src/lib.rs extensions/blinc_platform_desktop/tests/support.rs extensions/blinc_platform_desktop/tests/ime_runtime.rs
git commit -m "feat: add desktop ime event handling"
```

### Task 4: Teach Layout Widgets About Composition And Semantics

**Files:**
- Create: `crates/blinc_layout/src/accessibility.rs`
- Modify: `crates/blinc_layout/src/tests/mod.rs`
- Modify: `crates/blinc_layout/src/widgets/text_input.rs`
- Modify: `crates/blinc_layout/src/widgets/text_area.rs`
- Modify: `crates/blinc_layout/src/widgets/button.rs`
- Modify: `crates/blinc_layout/src/widgets/checkbox.rs`
- Modify: `crates/blinc_layout/src/tree.rs`
- Modify: `crates/blinc_layout/src/lib.rs`
- Create: `crates/blinc_layout/src/tests/accessibility_semantics.rs`

**Step 1: Write the failing layout tests**

- Execute this task only after Task 4 of `2026-03-06-desktop-file-flows-impl-plan.md` has landed, because that task creates the shared `blinc_layout` test harness.
- Reuse the `#[cfg(test)] mod tests;` hook introduced by the file-flows plan.
- Extend the existing `crates/blinc_layout/src/tests/mod.rs` aggregator introduced by the file-flows plan to wire in `accessibility_semantics`.
- Add a test named `accessibility_semantics_preserve_preedit_text`.
- Add a test named `accessibility_semantics_export_roles`.
- Add a test named `accessibility_semantics_focus_order`.

**Step 2: Run targeted tests to verify failure**

Run:
- `cargo test -p blinc_layout accessibility_semantics_export_roles -- --nocapture`

Expected: failures because layout widgets do not currently track composition or emit semantic snapshots.

**Step 3: Implement the minimal widget integration**

- Store composition state in text input and text area widgets.
- Add semantic metadata hooks to common interactive widgets.
- Add a layout-side snapshot exporter that can be consumed by the desktop accessibility backend.

**Step 4: Re-run targeted layout tests**

Run:
- `cargo test -p blinc_layout accessibility_semantics_export_roles -- --nocapture`

Expected: PASS.

**Step 5: Commit the widget integration**

```bash
git add crates/blinc_layout/src/accessibility.rs crates/blinc_layout/src/widgets/text_input.rs crates/blinc_layout/src/widgets/text_area.rs crates/blinc_layout/src/widgets/button.rs crates/blinc_layout/src/widgets/checkbox.rs crates/blinc_layout/src/tree.rs crates/blinc_layout/src/lib.rs crates/blinc_layout/src/tests/mod.rs crates/blinc_layout/src/tests/accessibility_semantics.rs
git commit -m "feat: add layout accessibility and ime support"
```
