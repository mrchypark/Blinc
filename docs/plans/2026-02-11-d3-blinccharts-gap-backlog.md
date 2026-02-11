# D3 Gap Closure Backlog for BlincCharts

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the D3-vs-BlincCharts capability matrix into a prioritized, phase-based backlog that supports repository split decisions.

**Architecture:** Keep the default track as product-parity (Blinc-native chart library), then add optional library-parity phases. Preserve current chart rendering/event model (`blinc_charts` + `blinc_layout` + `blinc_core`) and avoid foundation-package churn unless a concrete requirement forces it.

**Tech Stack:** Rust workspace crates (`blinc_charts`, `blinc_layout`, `blinc_core`), docs under `docs/references` and `docs/plans`.

---

## Source of Truth

- Capability matrix: `/Users/cypark/Documents/project/Blinc/docs/references/2026-02-11-d3-blinccharts-capability-matrix.md`
- Chart module surface: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/lib.rs`
- Gallery coverage: `/Users/cypark/Documents/project/Blinc/crates/blinc_app/examples/charts_gallery_demo.rs`

## Prioritization Rubric

- `P0`: split blocking or correctness credibility issue.
- `P1`: core developer-facing API gap for reusable chart library.
- `P2`: algorithmic depth and ecosystem parity improvements.
- `P3`: polish and optional parity layers.

## Package Change Policy (for split)

- Default policy: changes should remain in `blinc_charts`.
- Allowed optional touch points:
  - `blinc_layout`: only if new event primitives are strictly required.
  - `blinc_core`: only if new draw primitive API is strictly required.
- Any item that requires `blinc_layout` or `blinc_core` changes must carry explicit justification in its PR description.

## Phase 0 (P0): Split Credibility Blockers

Target window: immediate (before repo split announcement).

1. Replace `Parallel` stub in polar chart with real parallel coordinates rendering.
- Priority: `P0`
- D3 mapping: `d3-shape` / partial `d3-scale` parity credibility
- Current gap evidence: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/polar.rs`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - `PolarChartMode::Parallel` no longer calls `render_parallel_stub`.
  - Distinct axes + polyline rendering path exists.
  - Gallery mode toggle visibly switches to true parallel visualization.

2. Publish explicit support policy for `Full/Partial/Missing/Out-of-scope`.
- Priority: `P0`
- D3 mapping: all modules (contract clarity)
- Package impact: docs only
- Done criteria:
  - Matrix status rubric is referenced from README/docs entry point.
  - Split decision references product-parity target explicitly.

## Phase 1 (P1): Core Reusable Utility Layer

Target window: first cycle after split (if library consumers are expected).

3. Add reusable axis/tick module for Cartesian charts.
- Priority: `P1`
- D3 mapping: `d3-axis`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - Shared axis renderer used by at least line/bar/scatter.
  - Tick generation configurable by count and formatter callback.

4. Add scale abstraction beyond raw domain mapping.
- Priority: `P1`
- D3 mapping: `d3-scale`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - Introduce scale traits/types for linear and band scales.
  - Existing chart code can opt into shared scales incrementally.

5. Add formatting utilities for numeric and time labels.
- Priority: `P1`
- D3 mapping: `d3-format`, `d3-time-format`, partial `d3-time`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - Shared number formatter API replaces ad-hoc overlay text formatting.
  - Shared time label formatting usable in time-series axes/tooltips.

## Phase 2 (P2): Algorithmic Depth

Target window: second cycle after split (advanced analytics use cases).

6. Add spatial indexing utilities for dense interaction.
- Priority: `P2`
- D3 mapping: `d3-quadtree`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - Reusable quadtree-like index for nearest-point queries.
  - Scatter/network hover hit-testing can optionally use this index.

7. Add triangulation-based utilities for field/mesh workflows.
- Priority: `P2`
- D3 mapping: `d3-delaunay`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - New utility module with tests for finite input and stable topology.
  - At least one chart mode consumes the triangulation result.

8. Add polygon utility helpers for clipping/containment.
- Priority: `P2`
- D3 mapping: `d3-polygon`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - Point-in-polygon and area-like helpers exposed.
  - At least one brush/selection path uses shared polygon helper.

## Phase 3 (P3): Transition and Interpolation Layer

Target window: optional, based on UX goals.

9. Add interpolation helper module.
- Priority: `P3`
- D3 mapping: `d3-interpolate`
- Package impact: `blinc_charts` only (expected)
- Done criteria:
  - Reusable interpolation helpers available for chart value/path transitions.

10. Add chart transition API policy (minimal, deterministic).
- Priority: `P3`
- D3 mapping: `d3-transition`
- Package impact: may require `blinc_layout` integration; keep optional
- Done criteria:
  - Explicit transition lifecycle API is documented.
  - At least one chart demonstrates data-change animation with bounded cost.

## Explicit Out-of-Scope (for BlincCharts crate)

- `d3-fetch`, `d3-dsv`, `d3-selection`, `d3-timer`, `d3-ease`, `d3-random`
- Rationale: these are runtime/data/DOM/timing helpers, not core chart-rendering responsibilities for this crate.

## Split Decision Gates

Gate A (must-pass for split):
- Phase 0 complete.
- Current chart tests remain green: `cargo test -p blinc_charts`.
- Gallery compiles: `cargo check -p blinc_app --example charts_gallery_demo --features windowed`.

Gate B (optional quality bar before public externalization):
- At least Phase 1 items 3 and 4 complete.

## Recommended Execution Order

1. Phase 0 item 1 (Parallel real implementation).
2. Phase 0 item 2 (support policy visibility).
3. Phase 1 item 3 (axis/tick).
4. Phase 1 item 4 (scale abstraction).
5. Phase 1 item 5 (format/time-format).
6. Phase 2 and 3 based on external consumer demand.
