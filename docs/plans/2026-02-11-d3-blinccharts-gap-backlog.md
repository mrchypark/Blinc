# D3 Gap Closure for BlincCharts Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert the D3-vs-BlincCharts audit into an execution-ready backlog that protects split readiness while keeping scope aligned with Blinc-native chart priorities.

**Architecture:** Keep the default track as product-parity for `blinc_charts` and gate any broader library-parity work behind explicit demand. Implement split blockers first (`Parallel` mode credibility + support policy publication), then build reusable utility layers (axis, scale, formatting), then algorithmic modules. Avoid touching `blinc_layout` and `blinc_core` unless a concrete API constraint is proven.

**Tech Stack:** Rust workspace crates (`blinc_charts`, `blinc_app`, optional `blinc_layout`), documentation under `docs/references` and `docs/plans`, cargo test/check pipeline.

---

## Source of Truth

- Capability matrix: `/Users/cypark/Documents/project/Blinc/docs/references/2026-02-11-d3-blinccharts-capability-matrix.md`
- Chart module surface: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/lib.rs`
- Gallery coverage: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/examples/charts_gallery_demo.rs`
- Parallel stub evidence: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/polar.rs`

## Prioritization Rubric

- `P0`: split-blocking credibility or contract clarity issue.
- `P1`: core reusable chart-library API gap.
- `P2`: algorithmic depth and heavy-data interaction improvements.
- `P3`: optional interpolation/transition parity layer.

## Package Change Policy (for split)

- Default: keep all work inside `blinc_charts`.
- Optional touch points:
  - `blinc_layout`: only when transition/event primitives are strictly required.
  - `blinc_core`: only when drawing primitives are strictly required.
- If a task needs `blinc_layout` or `blinc_core`, PR must include explicit justification.

## Backlog Ledger

| ID | Priority | Scope | Dependencies | Status | Done criteria |
|---|---|---|---|---|---|
| `BLC-D3-001` | P0 | Real parallel coordinates rendering for `PolarChartMode::Parallel` | none | Completed (initial, 2026-02-11) | `render_parallel_stub` removed, distinct axes + polyline renderer added, gallery mode visibly differs from radar |
| `BLC-D3-002` | P0 | Publish support policy for `Full/Partial/Missing/Out-of-scope` | none | Completed (2026-02-11) | matrix rubric linked from entry docs and split recommendation |
| `BLC-D3-003` | P1 | Reusable axis/tick module for Cartesian charts | `BLC-D3-001` | Completed (initial, 2026-02-11) | line/bar/scatter share axis renderer, tick count + formatter configurable |
| `BLC-D3-004` | P1 | Shared scale abstraction (linear + band first) | `BLC-D3-003` recommended | Completed (initial, 2026-02-11) | chart modules can migrate incrementally from ad-hoc mapping |
| `BLC-D3-005` | P1 | Number/time formatting utilities | `BLC-D3-003`, `BLC-D3-004` | Completed (initial, 2026-02-11) | shared formatter API used in overlays/tooltips/axes |
| `BLC-D3-006` | P2 | Spatial index for dense hover/hit test | `BLC-D3-004` | Completed (initial, 2026-02-11) | optional index path for nearest-point queries in scatter/network |
| `BLC-D3-007` | P2 | Triangulation utility module | `BLC-D3-006` optional | Completed (initial, 2026-02-11) | finite-input tests + at least one consumer mode |
| `BLC-D3-008` | P2 | Polygon utility helpers | none | Completed (initial, 2026-02-11) | shared point-in-polygon/area helpers wired into selection path |
| `BLC-D3-009` | P3 | Interpolation helper module | `BLC-D3-003`~`005` recommended | Completed (initial, 2026-02-11) | reusable interpolation API for chart transitions |
| `BLC-D3-010` | P3 | Minimal deterministic transition policy | `BLC-D3-009` | Completed (initial, 2026-02-11) | documented lifecycle + one bounded-cost chart animation demo |

## Phase Plan

### Phase 0 (Split Gate)

- Target: immediate (before split announcement)
- Required items: `BLC-D3-001`, `BLC-D3-002`
- Required verification:
  - `cargo test -p blinc_charts`
  - `cargo check -p blinc_app --example charts_gallery_demo --features windowed`

### Phase 1 (Core Library Surface)

- Target: first cycle after split
- Required items: `BLC-D3-003`, `BLC-D3-004`, `BLC-D3-005`
- Suggested verification:
  - `cargo test -p blinc_charts`
  - `cargo test -p blinc_charts --test gallery_completion_smoke`
  - `cargo check -p blinc_app --example charts_gallery_demo --features windowed`

### Phase 2 (Algorithm Depth)

- Target: second cycle after split
- Items: `BLC-D3-006`, `BLC-D3-007`, `BLC-D3-008`

### Phase 3 (Optional Transition Layer)

- Target: only if UX animation goals justify maintenance cost
- Items: `BLC-D3-009`, `BLC-D3-010`

## Execution-Ready Tasks (Now)

### Task 1: `BLC-D3-001` Parallel Coordinates Real Implementation

**Files:**
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/polar.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/examples/charts_gallery_demo.rs` (labels/help text only if needed)
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/tests/gallery_completion_smoke.rs`

**Step 1: Write failing tests for `Parallel` behavior**

- Add tests proving `PolarChartMode::Parallel` does not use radar-only rendering path semantics (geometry/label expectations).

**Step 2: Run tests to verify failure**

- Run: `cargo test -p blinc_charts --test gallery_completion_smoke`
- Expected: fail on new `Parallel` assertions.

**Step 3: Implement minimal parallel coordinates renderer**

- Replace `render_parallel_stub` with dedicated render path:
  - equally spaced vertical axes by dimension
  - per-series polylines across axes
  - preserve finite-value handling + color rules

**Step 4: Re-run tests and fix until pass**

- Run: `cargo test -p blinc_charts --test gallery_completion_smoke`
- Expected: pass.

**Step 5: Verify gallery compile path**

- Run: `cargo check -p blinc_app --example charts_gallery_demo --features windowed`
- Expected: pass.

### Task 2: `BLC-D3-002` Support Policy Publication

**Files:**
- Modify: `/Users/cypark/Documents/project/Blinc/docs/references/2026-02-11-d3-blinccharts-capability-matrix.md`
- Modify: `/Users/cypark/Documents/project/Blinc/docs/plans/2026-02-11-d3-blinccharts-gap-backlog.md`
- Modify: `/Users/cypark/Documents/project/Blinc/README.md` (short pointer section)

**Step 1: Add visible rubric pointer from README**

- Add a concise section linking matrix + backlog policy.

**Step 2: Keep matrix and backlog IDs synchronized**

- Ensure backlog IDs (`BLC-D3-001`..`010`) are used consistently.

**Step 3: Verify docs references**

- Run: `rg -n "BLC-D3-00[1-9]|BLC-D3-010|Full/Partial/Missing/Out-of-scope" docs README.md`
- Expected: IDs and rubric strings found in matrix/backlog/readme.

### Task 3: `BLC-D3-003` Reusable Axis/Tick Module (Phase 1 first)

**Files:**
- Create: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/axis.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/lib.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/line.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/bar.rs`
- Modify: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/scatter.rs`
- Test: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/tests/gallery_completion_smoke.rs` (or new dedicated axis tests)

**Step 1: Add axis API and failing tests**

- Introduce tick generation contract (count + formatter callback).

**Step 2: Implement minimal axis renderer and integrate one chart first**

- Start with line chart, then port bar/scatter.

**Step 3: Verify chart compile and tests**

- Run: `cargo test -p blinc_charts`
- Run: `cargo check -p blinc_app --example charts_gallery_demo --features windowed`

## Explicit Out-of-Scope for `blinc_charts`

- `d3-fetch`, `d3-dsv`, `d3-selection`, `d3-timer`, `d3-ease`, `d3-random`
- Rationale: runtime/data/DOM/timing helpers are outside this crate’s rendering responsibility.

## Recommended Execution Order

1. `BLC-D3-001` (parallel credibility blocker)
2. `BLC-D3-002` (support policy visibility)
3. `BLC-D3-003` (axis/tick)
4. `BLC-D3-004` (scale abstraction)
5. `BLC-D3-005` (format/time-format)
6. `BLC-D3-006`..`010` by external consumer demand
