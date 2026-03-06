# Desktop Platform Master Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver the full desktop-platform expansion as a sequence of smaller executable plans with stable verification gates between phases.

**Architecture:** Keep cross-platform API contracts in `crates/blinc_platform`, put concrete desktop behavior in `extensions/blinc_platform_desktop`, and only touch `blinc_layout` / `blinc_app` where desktop features must surface into widgets or app runtime. Any shared trait widening must land with compile-safe no-op or default-preserving follow-ups for Android, iOS, and Harmony in the same task. Execute plans in dependency order so later shell/accessibility work builds on a stable windowing core.

**Tech Stack:** Rust 2021, `winit`, crate integration tests, desktop backend tests, `cargo fmt`, targeted `cargo test`.

---

### Task 1: Establish Baseline Before Any Desktop Expansion

**Files:**
- Modify: none
- Test: `crates/blinc_platform/tests/services_api.rs`

**Step 1: Run the current platform test baseline**

Run:
- `cargo test -p blinc_platform --test services_api -- --nocapture`
- `cargo test -p blinc_platform --test sensors_api -- --nocapture`

Expected: both test targets pass so later failures are attributable to desktop-platform changes.

**Step 2: Run the current desktop backend baseline**

Run:
- `cargo test -p blinc_platform_desktop -- --nocapture`

Expected: either zero tests or an all-pass run with no failing desktop backend assertions.

**Step 3: Record the display-less GUI test strategy up front**

- On Linux CI, wrap all `blinc_platform_desktop` test invocations with `xvfb-run --auto-servernum` for consistency.
- For platform-specific runtime tests that still require a real GUI session, reuse a shared `extensions/blinc_platform_desktop/tests/support.rs` `requires_display()` helper created in the first desktop runtime-test plan and skip cleanly when the environment is headless.
- Prefer mock-backed adapter tests for clipboard, tray, notifications, updater, and capture integrations unless the code path strictly requires a real native window.

**Step 4: Commit the verified baseline note**

```bash
git add .omx/context/desktop-platform-2026-03-06-205853.md .omx/interviews/desktop-platform-2026-03-06-205853.md .omx/specs/deep-interview-desktop-platform.md docs/plans/2026-03-06-desktop-platform-master-plan.md
git commit -m "docs: add desktop platform planning baseline"
```

### Task 2: Execute The Dependency Chain In Order

**Files:**
- Modify: `docs/plans/2026-03-06-desktop-windowing-foundation-impl-plan.md`
- Modify: `docs/plans/2026-03-06-desktop-file-flows-impl-plan.md`
- Modify: `docs/plans/2026-03-06-desktop-shell-integration-impl-plan.md`
- Modify: `docs/plans/2026-03-06-desktop-accessibility-ime-impl-plan.md`
- Modify: `docs/plans/2026-03-06-desktop-productization-impl-plan.md`

**Step 1: Execute the foundation plan first**

Run the tasks in:
- `docs/plans/2026-03-06-desktop-windowing-foundation-impl-plan.md`

Expected: shared window contracts and desktop implementation land with tests before higher-level integrations start.

**Step 2: Execute feature plans in this order**

Run next:
1. `docs/plans/2026-03-06-desktop-file-flows-impl-plan.md`
2. `docs/plans/2026-03-06-desktop-shell-integration-impl-plan.md`
3. `docs/plans/2026-03-06-desktop-accessibility-ime-impl-plan.md`
4. `docs/plans/2026-03-06-desktop-productization-impl-plan.md`

Expected: each plan rebases on already-tested abstractions instead of inventing parallel APIs.

### Task 3: Run End-To-End Verification After The Last Plan

**Files:**
- Modify only if verification exposes issues.

**Step 1: Run formatting**

Run:
- `cargo fmt --all`
- `cargo fmt --all -- --check`

Expected: formatting passes cleanly.

**Step 2: Run shared and desktop tests**

Run:
- `cargo test -p blinc_platform -- --nocapture`
- `cargo check -p blinc_platform_android`
- `cargo check -p blinc_platform_ios`
- `cargo check -p blinc_platform_harmony`
- `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop -- --nocapture`

Expected: shared API tests, non-desktop compile checks, and desktop backend tests pass together.

**Step 3: Run representative integration tests for consumers**

Run:
- `cargo test -p blinc_layout --lib -- --nocapture`
- `cargo test -p blinc_app --features windowed --lib -- --nocapture`

Expected: widget/runtime consumers still compile and pass after desktop API expansion.

**Step 4: Commit the integrated desktop platform upgrade**

```bash
git add crates/blinc_platform crates/blinc_layout crates/blinc_app extensions/blinc_platform_desktop docs/plans
git commit -m "feat: expand desktop platform capabilities"
```
