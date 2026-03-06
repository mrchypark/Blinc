# Card Text Wrap Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make wrapping text inside cards and headers receive a stable width, add paragraph-friendly width APIs on `Text`, and emit diagnostics when wrap measurement happens without a definite width.

**Architecture:** Fix the issue at the layout contract boundary instead of the renderer. Card-like block containers should stretch children by default, `Text` should preserve explicit width constraints across builder mutations, and the Taffy text measure callback should log when it has to fall back to intrinsic single-line measurement.

**Tech Stack:** Rust 2021, Taffy flex layout, `tracing`, crate-local unit tests.

---

### Task 1: Add Red Tests For Card Wrap Regressions

**Files:**
- Modify: `crates/blinc_cn/src/components/card.rs`

**Step 1: Write the failing tests**

- Add a test proving a long text child wraps inside `card().w(120.0)`.
- Add a test proving `card_header().description(long_text)` wraps inside the header width.

**Step 2: Run targeted tests to verify failure**

Run: `cargo test -p blinc_cn card::tests:: -- --nocapture`

Expected: at least one wrap assertion fails because text remains single-line or wider than the card/header.

### Task 2: Add Red Tests For Text Width API

**Files:**
- Modify: `crates/blinc_layout/src/text.rs`

**Step 1: Write the failing tests**

- Add a test for `text(...).w_full().size(16.0)` preserving `width = 100%`.
- Add a test for `text(...).max_w(180.0).size(16.0)` preserving `max_width = 180px`.

**Step 2: Run targeted tests to verify failure**

Run: `cargo test -p blinc_layout text::tests:: -- --nocapture`

Expected: tests fail to compile or fail assertions because the width helpers do not exist yet.

### Task 3: Implement Minimal Layout Fixes

**Files:**
- Modify: `crates/blinc_cn/src/components/card.rs`
- Modify: `crates/blinc_layout/src/text.rs`
- Modify: `crates/blinc_layout/src/tree.rs`

**Step 1: Update card containers**

- Change `Card::new()` to use `items_stretch()`.
- Change `CardHeader::new()` to use `items_stretch()` so title/description text inherits width by default.

**Step 2: Add `Text` width helpers**

- Introduce stored width/max-width overrides on `Text`.
- Add `w`, `w_full`, `w_auto`, and `max_w`.
- Make `update_size_estimate()` respect those overrides for both wrapping and non-wrapping text.

**Step 3: Add measure fallback diagnostics**

- In `text_measure_function`, prefer known width, available definite width, and fixed style hints.
- Emit `tracing::debug!` when wrapping text still has no definite width.

### Task 4: Verify, Format, And Re-Run

**Files:**
- Modify only if verification reveals gaps.

**Step 1: Run targeted tests**

Run:
- `cargo test -p blinc_cn card::tests:: -- --nocapture`
- `cargo test -p blinc_layout text::tests:: -- --nocapture`

**Step 2: Run formatting**

Run:
- `cargo fmt --all`
- `cargo fmt --all -- --check`

**Step 3: Run broader verification**

Run:
- `cargo test -p blinc_cn`
- `cargo test -p blinc_layout`
