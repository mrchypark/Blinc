# iOS Runtime Surface Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the most visible placeholder behavior in `blinc_platform_ios` with UIKit-backed environment reads and a truthful UIKit-managed event-loop surface.

**Architecture:** Keep public APIs stable. Add small internal query helpers in `app.rs` for dark mode and safe area, then update `IOSEventLoop::run()` to behave like a UIKit-managed bridge surface rather than a missing feature. Prefer helper extraction so non-iOS tests can verify fallback logic without simulator execution.

**Tech Stack:** Rust 1.75, edition 2021, `objc2`, `objc2-ui-kit`, `blinc_platform_ios`.

---

### Task 1: Add iOS Environment Query Helpers

**Files:**
- Modify: `extensions/blinc_platform_ios/src/app.rs`
- Test: `extensions/blinc_platform_ios/src/app.rs`

**Step 1: Extract helpers**

Add internal helpers that return:

- optional dark-mode state
- optional safe-area insets

UIKit code should stay behind `#[cfg(target_os = "ios")]`.

**Step 2: Keep public fallback behavior**

Have `is_dark_mode()` and `get_safe_area_insets()` call the helpers and fall back
when UIKit context is unavailable.

**Step 3: Add focused tests**

Add tests for the fallback wrappers/helper combinators that run on non-iOS too.

### Task 2: Replace Unsupported iOS Event Loop Behavior

**Files:**
- Modify: `extensions/blinc_platform_ios/src/event_loop.rs`
- Test: `extensions/blinc_platform_ios/src/event_loop.rs`

**Step 1: Adjust run semantics**

Make `IOSEventLoop::run()` dispatch a minimal set of bridge-safe events and
return `Ok(())` instead of `Unsupported`.

**Step 2: Add focused tests**

On non-iOS targets, keep placeholder behavior as-is. Add tests for any shared
event-sequence helper used by the iOS path.

### Task 3: Update iOS Surface Docs

**Files:**
- Modify: `extensions/blinc_platform_ios/README.md`

**Step 1: Clarify runtime guarantees**

Document that:

- environment queries now reflect UIKit when available
- the event loop surface is UIKit-managed rather than Rust-owned
- renderer/lifecycle completeness still depends on the host app

### Task 4: Verification

**Step 1: Run formatting**

Run: `cargo fmt --all`

**Step 2: Run formatting check**

Run: `cargo fmt --all -- --check`

**Step 3: Run tests**

Run: `cargo test -p blinc_platform_ios`
