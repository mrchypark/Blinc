# Desktop File Flows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add desktop-native file dialogs, drag-and-drop file intake, and richer clipboard support so Blinc apps can participate in ordinary desktop workflows.

**Architecture:** Define shared dialog and data-transfer contracts in `blinc_platform`, first refactor clipboard access so public APIs can dispatch to a desktop backend instead of only `native_bridge`, then extend the input/event surface for drag-and-drop, and finally implement desktop-native behavior in `blinc_platform_desktop` using backend-specific helpers. This plan owns creation of the shared `blinc_layout` test harness (`#[cfg(test)] mod tests;` plus `src/tests/mod.rs`) that later desktop plans will extend.

**Tech Stack:** Rust 2021, `winit`, native dialog crate (`rfd` or equivalent), desktop integration tests.

---

### Task 1: Add Red Tests For Dialog And Drag-Drop APIs

**Files:**
- Create: `crates/blinc_platform/tests/file_flows_api.rs`
- Modify: `crates/blinc_platform/src/input.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Write the failing tests**

- Add a test for a `FileDialogOptions` builder producing open/save/directory selection requests.
- Add a test for `InputEvent` carrying `DragEntered`, `DragMoved`, `DragLeft`, and `DropFiles` payloads.
- Add a test for clipboard payload types supporting plain text plus file URI lists.

**Step 2: Run targeted tests to verify failure**

Run:
- `cargo test -p blinc_platform --test file_flows_api -- --nocapture`

Expected: missing modules and enum variants cause compile failures.

**Step 3: Commit the red tests**

```bash
git add crates/blinc_platform/tests/file_flows_api.rs crates/blinc_platform/src/input.rs crates/blinc_platform/src/lib.rs
git commit -m "test: add desktop file flow contracts"
```

### Task 2: Add Shared Dialog And Clipboard Contracts

**Files:**
- Create: `crates/blinc_platform/src/dialogs.rs`
- Modify: `crates/blinc_platform/src/clipboard.rs`
- Create: `crates/blinc_platform/src/clipboard_backend.rs`
- Modify: `crates/blinc_platform/src/input.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Add a dialog abstraction**

- Introduce `FileDialogKind`, `FileDialogOptions`, and `FileDialogResult`.
- Keep the API synchronous at first to minimize blast radius.

**Step 2: Introduce clipboard backend dispatch**

- Add a `ClipboardBackend` trait plus process-local registry in `crates/blinc_platform/src/clipboard_backend.rs`.
- Keep the current `native_bridge` path as the default backend so mobile/service tests continue to pass.
- Refactor `crates/blinc_platform/src/clipboard.rs` to dispatch through the registry before falling back to `native_bridge`.

**Step 3: Extend clipboard types**

- Add typed clipboard payloads for `Text`, `Html`, `FileList`, and an `Unsupported` fallback.
- Preserve current text helper functions as a convenience layer on top of the typed API.

**Step 4: Extend drag-and-drop input events**

- Add file hover and drop events to `InputEvent`.
- Ensure events carry absolute paths and cursor position.

**Step 5: Re-run shared API tests**

Run:
- `cargo test -p blinc_platform --test file_flows_api -- --nocapture`

Expected: shared contracts compile and tests pass.

**Step 6: Commit the shared file-flow layer**

```bash
git add crates/blinc_platform/src/dialogs.rs crates/blinc_platform/src/clipboard.rs crates/blinc_platform/src/clipboard_backend.rs crates/blinc_platform/src/input.rs crates/blinc_platform/src/lib.rs crates/blinc_platform/tests/file_flows_api.rs
git commit -m "feat: add shared desktop file flow abstractions"
```

### Task 3: Implement Desktop Dialogs, Clipboard Payloads, And Drag-Drop

**Files:**
- Create: `extensions/blinc_platform_desktop/src/dialogs.rs`
- Create: `extensions/blinc_platform_desktop/src/clipboard.rs`
- Modify: `extensions/blinc_platform_desktop/src/event_loop.rs`
- Modify: `extensions/blinc_platform_desktop/src/input.rs`
- Modify: `extensions/blinc_platform_desktop/src/lib.rs`
- Modify: `extensions/blinc_platform_desktop/Cargo.toml`
- Create: `extensions/blinc_platform_desktop/tests/file_flows_runtime.rs`
- Modify: `extensions/blinc_platform_desktop/tests/support.rs`

**Step 1: Write the failing backend tests**

- Add a test for translating `winit` dropped-file callbacks into the new `InputEvent` variants.
- Add a test double for dialog execution returning deterministic file paths.
- Add a test proving `DesktopPlatform::new()` or `create_event_loop_with_config()` registers the clipboard backend before the first public clipboard call.
- Add a test for typed clipboard read/write on platforms that expose it in CI, with a skip guard if the environment lacks a display server.

**Step 2: Run targeted backend tests to verify failure**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test file_flows_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test file_flows_runtime -- --nocapture`

Expected: failures because the backend has no dialog module and no file-drop event mapping.

**Step 3: Implement desktop integrations**

- Wire native open/save/select-folder dialogs behind `FileDialogOptions`.
- Add `ensure_desktop_clipboard_backend_registered()` in `extensions/blinc_platform_desktop/src/clipboard.rs`.
- Call that registration helper from `DesktopPlatform::new()` and `create_event_loop_with_config()` in `extensions/blinc_platform_desktop/src/lib.rs` so public `blinc_platform::clipboard` functions can reach the new backend before any windowed app code touches the clipboard.
- Convert `HoveredFile`, `DroppedFile`, and cancellation events from `winit`.
- Provide typed clipboard translation, falling back to text-only where richer payloads are unavailable.

**Step 4: Re-run backend and shared tests**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test file_flows_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test file_flows_runtime -- --nocapture`
- `cargo test -p blinc_platform --test file_flows_api -- --nocapture`

Expected: both pass.

**Step 5: Commit the desktop file-flow implementation**

```bash
git add extensions/blinc_platform_desktop/src/dialogs.rs extensions/blinc_platform_desktop/src/clipboard.rs extensions/blinc_platform_desktop/src/event_loop.rs extensions/blinc_platform_desktop/src/input.rs extensions/blinc_platform_desktop/src/lib.rs extensions/blinc_platform_desktop/tests/support.rs extensions/blinc_platform_desktop/tests/file_flows_runtime.rs extensions/blinc_platform_desktop/Cargo.toml
git commit -m "feat: add desktop dialogs and drag-drop flows"
```

### Task 4: Add A Consumer Smoke Test In Layout

**Files:**
- Modify: `crates/blinc_layout/src/event_router.rs`
- Create: `crates/blinc_layout/src/tests/mod.rs`
- Create: `crates/blinc_layout/src/tests/desktop_file_flows.rs`
- Modify: `crates/blinc_layout/src/lib.rs`

**Step 1: Write the failing smoke test**

- Add `#[cfg(test)] mod tests;` to `crates/blinc_layout/src/lib.rs`.
- Create `crates/blinc_layout/src/tests/mod.rs` and wire in `desktop_file_flows`.
- Add a test named `desktop_file_flows_routes_events` proving the layout event router does not discard the new drag-and-drop events.

**Step 2: Run the targeted smoke test to verify failure**

Run:
- `cargo test -p blinc_layout desktop_file_flows_routes_events -- --nocapture`

Expected: the event router ignores the new variants or the test module is missing.

**Step 3: Make the minimal integration change**

- Update routing matches to accept and forward the new file flow events without introducing widget behavior yet.

**Step 4: Re-run the smoke test**

Run:
- `cargo test -p blinc_layout desktop_file_flows_routes_events -- --nocapture`

Expected: PASS.

**Step 5: Commit the consumer integration**

```bash
git add crates/blinc_layout/src/event_router.rs crates/blinc_layout/src/lib.rs crates/blinc_layout/src/tests/mod.rs crates/blinc_layout/src/tests/desktop_file_flows.rs
git commit -m "feat: route desktop file flow events through layout"
```
