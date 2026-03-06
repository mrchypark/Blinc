# Desktop Windowing Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand the shared window/event abstraction and desktop backend so Blinc can behave like a real desktop shell rather than a fixed-size demo window.

**Architecture:** First widen `blinc_platform` with explicit window state/value objects and richer events, then add compile-safe no-op/default follow-ups for Android, iOS, and Harmony, then implement the richer behavior in `blinc_platform_desktop` on top of `winit`, and finally lock behavior with shared API tests plus desktop runtime tests. GUI runtime tests must use a shared display probe and Linux `xvfb-run` wrapper instead of assuming a visible desktop session.

**Tech Stack:** Rust 2021, `winit`, crate integration tests, desktop backend tests.

---

### Task 1: Add Red Tests For Window Control Contracts

**Files:**
- Create: `crates/blinc_platform/tests/window_api.rs`
- Modify: `crates/blinc_platform/src/window.rs`
- Modify: `crates/blinc_platform/src/event.rs`

**Step 1: Write the failing tests**

- Add a test for `WindowConfig` carrying `min_size`, `max_size`, `position`, `icon_path`, and startup visibility.
- Add a test for new `WindowEvent` variants covering `Maximized`, `Minimized`, `Restored`, `Occluded`, and `MonitorChanged`.
- Add a test for a new `MonitorInfo` / `WindowBounds` value type roundtrip.

**Step 2: Run the targeted test to verify failure**

Run:
- `cargo test -p blinc_platform --test window_api -- --nocapture`

Expected: compile failures or missing-symbol failures because the richer contracts do not exist yet.

**Step 3: Commit the red tests**

```bash
git add crates/blinc_platform/tests/window_api.rs crates/blinc_platform/src/window.rs crates/blinc_platform/src/event.rs
git commit -m "test: add windowing foundation contract coverage"
```

### Task 2: Add Shared Window And Monitor Primitives

**Files:**
- Modify: `crates/blinc_platform/src/window.rs`
- Modify: `crates/blinc_platform/src/event.rs`
- Modify: `crates/blinc_platform/src/lib.rs`
- Modify: `extensions/blinc_platform_android/src/window.rs`
- Modify: `extensions/blinc_platform_ios/src/window.rs`
- Modify: `extensions/blinc_platform_harmony/src/window.rs`

**Step 1: Extend `WindowConfig` minimally**

- Add optional fields for minimum size, maximum size, startup position, visibility, and icon path.
- Add builder methods for each new field instead of ad hoc setters later.

**Step 2: Expand the `Window` trait**

- Add methods for `set_size`, `set_min_size`, `set_max_size`, `set_position`, `set_visible`, `set_always_on_top`, `set_fullscreen`, `minimize`, `maximize`, `restore`, `set_decorations`, and `monitors`.
- Introduce lightweight shared structs such as `WindowBounds`, `WindowPosition`, and `MonitorInfo`.

**Step 3: Expand `WindowEvent`**

- Add variants for maximize/minimize/restore/occlusion and monitor changes.
- Keep the enum additive so current consumers can update via exhaustive matches.

**Step 4: Add compile-safe follow-ups for non-desktop backends**

- Update Android, iOS, and Harmony `Window` implementations to satisfy the widened trait with no-op or best-effort behavior where desktop-only features do not apply.
- Preserve existing behavior by returning sensible defaults instead of faking desktop capabilities.

**Step 5: Re-run the contract tests and cross-platform compile checks**

Run:
- `cargo test -p blinc_platform --test window_api -- --nocapture`
- `cargo check -p blinc_platform_android`
- `cargo check -p blinc_platform_ios`
- `cargo check -p blinc_platform_harmony`

Expected: shared API tests pass and all non-desktop platform crates still compile before the desktop backend implements every behavior.

**Step 6: Commit the shared abstraction layer**

```bash
git add crates/blinc_platform/src/window.rs crates/blinc_platform/src/event.rs crates/blinc_platform/src/lib.rs crates/blinc_platform/tests/window_api.rs extensions/blinc_platform_android/src/window.rs extensions/blinc_platform_ios/src/window.rs extensions/blinc_platform_harmony/src/window.rs
git commit -m "feat: extend shared desktop window contracts"
```

### Task 3: Implement Desktop Window Controls And Monitor Support

**Files:**
- Modify: `extensions/blinc_platform_desktop/src/window.rs`
- Modify: `extensions/blinc_platform_desktop/src/event_loop.rs`
- Modify: `extensions/blinc_platform_desktop/src/lib.rs`
- Create: `extensions/blinc_platform_desktop/tests/window_runtime.rs`
- Create: `extensions/blinc_platform_desktop/tests/support.rs`
- Modify: `extensions/blinc_platform_desktop/Cargo.toml`

**Step 1: Write the failing desktop runtime tests**

- Add a test for applying min/max size and startup position from `WindowConfig`.
- Add a test for forwarding focus/scale/maximize lifecycle events into the shared `WindowEvent` model.
- Add a test for monitor enumeration returning at least the current monitor metadata when available.

**Step 2: Run the targeted desktop test to verify failure**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test window_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test window_runtime -- --nocapture`

Expected: failures because the desktop backend currently exposes only title, cursor, redraw, focus, and visibility.

**Step 3: Add a shared display guard for runtime tests**

- Create `extensions/blinc_platform_desktop/tests/support.rs` with a `requires_display()` helper.
- Skip GUI-runtime assertions cleanly when the environment cannot create a native window.

**Step 4: Implement `DesktopWindow` methods**

- Map new `Window` trait methods onto `winit::window::Window`.
- Store any derived state that `winit` cannot query directly after mutation.
- Respect `always_on_top`, fullscreen, visibility, and decorations from `WindowConfig`.

**Step 5: Implement event-loop forwarding**

- Translate `winit` maximize/minimize/occlusion/monitor movement signals into the shared `WindowEvent` variants.
- Request redraw only when the new state requires a render pass.

**Step 6: Re-run backend tests**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test window_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test window_runtime -- --nocapture`
- `cargo test -p blinc_platform --test window_api -- --nocapture`

Expected: both shared and desktop tests pass.

**Step 7: Commit the desktop backend foundation**

```bash
git add extensions/blinc_platform_desktop/src/window.rs extensions/blinc_platform_desktop/src/event_loop.rs extensions/blinc_platform_desktop/src/lib.rs extensions/blinc_platform_desktop/tests/support.rs extensions/blinc_platform_desktop/tests/window_runtime.rs extensions/blinc_platform_desktop/Cargo.toml
git commit -m "feat: implement desktop window control foundation"
```

### Task 4: Verify And Format Before Higher Layers

**Files:**
- Modify only if verification reveals gaps.

**Step 1: Run formatting**

Run:
- `cargo fmt --all`
- `cargo fmt --all -- --check`

Expected: formatting passes.

**Step 2: Run broader smoke coverage**

Run:
- `cargo test -p blinc_platform -- --nocapture`
- `cargo check -p blinc_platform_android`
- `cargo check -p blinc_platform_ios`
- `cargo check -p blinc_platform_harmony`
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop -- --nocapture`

Expected: no regressions in existing platform service tests, non-desktop crates still compile, and desktop runtime tests use the display-safe path.
