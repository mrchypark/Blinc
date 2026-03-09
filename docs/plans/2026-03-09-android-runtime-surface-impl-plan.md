# Android Runtime Surface Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the most visible Android JNI/activity placeholder behavior with a coherent runtime state model that preserves render, resize, focus, and touch state.

**Architecture:** Keep public JNI signatures stable. Introduce a pure Rust state object for JNI bridge transitions so host tests can verify behavior. Then align `BlincAndroidApp` lifecycle/renderability flags with that same bridge story. Do not claim a full GPU renderer exists.

**Tech Stack:** Rust 1.75, edition 2021, JNI bridge layer, Android activity integration.

---

### Task 1: Extract Android JNI Runtime State

**Files:**
- Modify: `extensions/blinc_platform_android/src/jni_bridge.rs`
- Test: `extensions/blinc_platform_android/src/jni_bridge.rs`

**Step 1: Introduce pure state model**

Add a target-independent internal state type that tracks:

- dimensions
- scale factor
- focus flag
- redraw requested / surface dirty flags
- touch-active flag
- last logical touch position
- queued logical touch events
- last rendered size

**Step 2: Delegate JNI transitions**

Have `nativeInit`, `nativeRenderFrame`, `nativeOnTouch`, and `nativeResize`
update this model instead of only logging.

**Step 3: Add unit tests**

Verify:

- init starts dirty
- resize requests redraw
- touch events are translated to logical coordinates and queued
- render clears pending redraw markers

### Task 2: Align Android Activity Renderability State

**Files:**
- Modify: `extensions/blinc_platform_android/src/activity.rs`
- Test: `extensions/blinc_platform_android/src/activity.rs`

**Step 1: Extend lifecycle state**

Track whether redraw is needed and mark it on init/resume/resize/focus events.
In practice, `Resume` should not force `focused = true`; redraw may be
re-requested on resume only when the app is already focused or when a later
focus-gain event arrives.

**Step 2: Make render path consume redraw need**

`render_frame()` should clear pending redraw markers even if the GPU path is not
implemented yet.

**Step 3: Add focused tests**

Verify `should_render()` and redraw semantics across focus/window transitions.

### Task 3: Verification

**Step 1: Run formatting**

Run: `cargo fmt --all`

**Step 2: Run formatting check**

Run: `cargo fmt --all -- --check`

**Step 3: Run tests**

Run:

```bash
cargo test -p blinc_platform_android
```
