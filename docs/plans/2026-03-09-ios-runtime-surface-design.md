# iOS Runtime Surface Design

**Scope:** `extensions/blinc_platform_ios` runtime surface only.

## Problem

The current iOS platform crate still exposes placeholder behavior in three places
that leak directly into the public API:

- `is_dark_mode()` always returns `false`
- `get_safe_area_insets()` always returns zeroes
- `IOSEventLoop::run()` returns `Unsupported`, even though the crate already
  documents UIKit-managed lifecycle integration

This creates a gap between what the crate surface suggests and what downstream
code can rely on.

## Goal

Make the iOS platform surface more truthful and immediately useful without
pretending Blinc owns the iOS runtime.

## Recommended Approach

### 1. Use UIKit for environment reads

Implement small iOS-only helpers that query:

- active interface style for dark mode
- active window/root-view safe area insets

These should stay inside `app.rs` so the public surface remains unchanged.

### 2. Separate query logic from fallback policy

Split UIKit access into internal helper functions that return `Option<_>`.
Public API functions should preserve the existing cross-platform fallback story:

- dark mode falls back to `false`
- safe area falls back to `(0, 0, 0, 0)`

This keeps behavior explicit and testable.

### 3. Reframe the event loop as UIKit-managed

`IOSEventLoop::run()` should no longer present itself as a missing feature.
Instead, it should:

- emit the minimal lifecycle/frame events that can be produced from the Rust side
- return `Ok(())`
- document that UIKit owns the real run loop and frame scheduling

This makes the API truthful: the event loop exists as an integration surface,
but it is not a desktop-style blocking owner loop.

## Non-Goals

- Building a full Rust-owned iOS run loop
- Implementing full CADisplayLink-driven event delivery inside the Rust crate
- Adding iOS integration tests that require simulator/device execution

## Testing

- Add unit tests for fallback/helper behavior that do not require iOS runtime
- Keep iOS-specific UIKit calls behind `#[cfg(target_os = "ios")]`
- Run `cargo test -p blinc_platform_ios`

