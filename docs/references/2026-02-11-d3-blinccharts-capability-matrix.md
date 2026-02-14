# D3 vs BlincCharts Capability Matrix (2026-02-11)

Purpose: compare the latest `d3` package capability surface with current `blinc_charts` so we can decide repo split readiness with evidence.

## Scope and Rubric

- D3 reference version: `d3@7.9.0` (npm latest on 2026-02-11).
- Blinc scope: `crates/blinc_charts` (public modules + gallery usage).
- Status rubric:
  - `Full`: equivalent capability exists in BlincCharts.
  - `Partial`: capability exists but with limited surface or different constraints.
  - `Missing`: no practical equivalent yet.
  - `Out-of-scope`: web/DOM utility not intended for BlincCharts runtime.

Key evidence:
- Public chart modules: `crates/blinc_charts/src/lib.rs`
- Gallery coverage (18 chart kinds wired): `crates/blinc_app/examples/charts_gallery_demo.rs`
- Shared view/domain mapping: `crates/blinc_charts/src/view.rs`
- Shared link/selection state: `crates/blinc_charts/src/link.rs`
- Shared interaction bindings: `crates/blinc_charts/src/input.rs`

## Capability Matrix (D3 30 modules)

| D3 module | Primary capability | BlincCharts mapping | Status | Evidence / note |
|---|---|---|---|---|
| `d3-array` | stats, bins, extents | histogram/statistics, domain computations | Partial | histogram/statistics exist, but not a general array utility API |
| `d3-axis` | reusable axis generators | shared axis/tick helpers in `axis.rs`, consumed by line/bar/scatter overlays | Partial | reusable module exists, but advanced axis layouts are still limited |
| `d3-brush` | 1D/2D brushing | `BrushX`, `BrushRect`, linked selection | Full | `brush.rs`, `xy_stack.rs`, `density_map.rs`, `contour.rs` |
| `d3-chord` | chord layout/rendering | `NetworkMode::Chord` | Partial | chord mode exists in `network.rs`, simplified pipeline |
| `d3-color` | color parsing/manipulation | `blinc_core::Color` usage in styles | Partial | chart styles use colors, but no d3-like color utility package |
| `d3-contour` | contour / isobands | `contour.rs` | Full | marching-squares style contour rendering implemented |
| `d3-delaunay` | delaunay/voronoi | lightweight triangulation helpers | Partial | `triangulation.rs` fan triangulation exists; full delaunay/voronoi not yet implemented |
| `d3-dispatch` | event dispatch bus | framework events + shared state handles | Partial | interactions are event callbacks, no standalone dispatch API |
| `d3-drag` | drag behavior | drag pan/brush in all interactive charts | Full | `on_drag*` patterns in line/bar/scatter/etc |
| `d3-dsv` | CSV/TSV parsing | none in `blinc_charts` | Out-of-scope | data ingestion is outside chart rendering crate |
| `d3-ease` | easing functions | none in `blinc_charts` | Out-of-scope | animation easing belongs to other runtime layers |
| `d3-fetch` | web fetch helpers | none in `blinc_charts` | Out-of-scope | network/data fetch is outside chart crate |
| `d3-force` | force simulation | graph layout mode in `network.rs` | Partial | graph exists, but no standalone configurable force engine API |
| `d3-format` | number formatting | shared numeric format helpers | Partial | `format.rs` (`format_fixed`, `format_compact`) used by chart overlays/axes |
| `d3-geo` | geo projections/pathing | `geo.rs` | Partial | geo chart exists; projection set is narrower than d3-geo |
| `d3-hierarchy` | tree/treemap/partition/pack | `hierarchy.rs` | Partial | Treemap/Icicle/Sunburst/Packing exist, narrower API surface |
| `d3-interpolate` | interpolation utilities | reusable interpolation helpers | Partial | `interpolate.rs` (`lerp_f32`, `lerp_point`) |
| `d3-path` | path serialization | `blinc_core::Path` drawing | Partial | path drawing exists, but not a d3-path-like utility package |
| `d3-polygon` | polygon ops | shared polygon helper module | Partial | `polygon.rs` (`point_in_polygon`, `polygon_area`, `rect_polygon`) wired into contour/density selection overlays |
| `d3-quadtree` | spatial index | shared spatial index helper module | Partial | `spatial_index.rs` grid index used for scatter/network nearest-hit lookups |
| `d3-random` | random distributions | demo-local generators only | Out-of-scope | random helpers are in gallery example, not chart API |
| `d3-scale` | scale primitives | `Domain1D/2D` + px/data transforms | Partial | linear mapping exists (`view.rs`), no band/log/time scale module |
| `d3-scale-chromatic` | color palettes | per-chart hardcoded palettes | Partial | colors exist, no palette package surface |
| `d3-selection` | DOM selections | n/a in Blinc UI runtime | Out-of-scope | Blinc uses element builders/events, not DOM selection |
| `d3-shape` | line/area/arc/symbol generators | line/area/bar/scatter/polar etc | Partial | many shape charts exist; no generic shape generator API |
| `d3-time` | time intervals/scales | numeric X domain time-series | Partial | time series support exists (`time_series.rs`), no rich calendar API |
| `d3-time-format` | locale time formatting | shared time-format helper module | Partial | `time_format.rs` (`format_hms`, `format_time_or_number`) used in Cartesian chart overlays/axes |
| `d3-timer` | frame timers | none in `blinc_charts` | Out-of-scope | scheduling runtime is outside chart crate |
| `d3-transition` | declarative transitions | deterministic value transition helper + gauge integration | Partial | `transition.rs` (`ValueTransition`) + gauge demo path |
| `d3-zoom` | pan/zoom behavior | scroll/pinch/drag pan across chart types | Full | widespread `on_scroll`/`on_pinch`/`on_drag_pan_total` |

## Backlog Mapping (Execution IDs)

Use these IDs in commit messages, PR descriptions, and progress notes so the capability matrix and backlog stay synchronized.

| Backlog ID | Priority | D3 module(s) | Work summary | Primary files |
|---|---|---|---|---|
| `BLC-D3-001` | P0 | `d3-shape` (parallel-coordinates credibility) | Replace `PolarChartMode::Parallel` stub with real parallel coordinates rendering | `crates/blinc_charts/src/polar.rs`, `crates/blinc_app/examples/charts_gallery_demo.rs`, `crates/blinc_charts/tests/gallery_completion_smoke.rs` |
| `BLC-D3-002` | P0 | All (status contract) | Publish and link `Full/Partial/Missing/Out-of-scope` support policy from entry docs | `docs/references/2026-02-11-d3-blinccharts-capability-matrix.md`, `docs/plans/2026-02-11-d3-blinccharts-gap-backlog.md`, `README.md` |
| `BLC-D3-003` | P1 | `d3-axis` | Add reusable axis/tick renderer consumed by line/bar/scatter | `crates/blinc_charts/src/axis.rs` (new), `crates/blinc_charts/src/lib.rs`, `crates/blinc_charts/src/line.rs`, `crates/blinc_charts/src/bar.rs`, `crates/blinc_charts/src/scatter.rs` |
| `BLC-D3-004` | P1 | `d3-scale` | Introduce shared linear/band scale abstractions | `crates/blinc_charts/src/scale.rs` (new), `crates/blinc_charts/src/view.rs`, `crates/blinc_charts/src/lib.rs` |
| `BLC-D3-005` | P1 | `d3-format`, `d3-time-format` | Add numeric/time formatting helpers for axis/tooltips/overlay text | `crates/blinc_charts/src/format.rs` (new), `crates/blinc_charts/src/time_format.rs` (new), chart modules with overlay labels |
| `BLC-D3-006` | P2 | `d3-quadtree` | Add optional spatial index for dense hit-testing | `crates/blinc_charts/src/spatial_index.rs` (new), `crates/blinc_charts/src/scatter.rs`, `crates/blinc_charts/src/network.rs` |
| `BLC-D3-007` | P2 | `d3-delaunay` | Add triangulation utilities and wire at least one chart mode | `crates/blinc_charts/src/triangulation.rs` (new), one consuming chart module |
| `BLC-D3-008` | P2 | `d3-polygon` | Add polygon helpers (containment/area) for shared selection logic | `crates/blinc_charts/src/polygon.rs` (new), brush/selection consumers |
| `BLC-D3-009` | P3 | `d3-interpolate` | Add interpolation utility module for deterministic transitions | `crates/blinc_charts/src/interpolate.rs` (new), chart animation consumers |
| `BLC-D3-010` | P3 | `d3-transition` | Define minimal transition lifecycle policy and sample integration | docs + selected chart module, optional `blinc_layout` touch |

## Transition Policy (Minimal, Deterministic)

- Lifecycle states:
  - `Idle`: no active transition (`transition = None`).
  - `Active`: `ValueTransition` exists and advances with `step(dt)`.
  - `Completed`: elapsed time reaches duration, value snaps to target, transition cleared.
- Determinism rule:
  - transitions use explicit `dt` progression (`tick_transition`) and no hidden scheduler/timer inside `blinc_charts`.
  - chart render path may apply a fixed per-frame step to keep cost bounded and predictable.
- Initial integration target:
  - `GaugeChartModel` uses `set_value_transition` + `tick_transition` as the reference implementation.
- Scope rule:
  - this crate provides interpolation/transition primitives; orchestration across multiple charts or timeline control remains optional and can be added by higher layers.

## BlincCharts coverage snapshot

- Implemented chart families (18 kinds wired in gallery): line, multi-line, area, bar, histogram, scatter, candlestick, heatmap, stacked area, density map, contour, statistics, hierarchy, network, polar, gauge, funnel, geo.
- X-domain linking + hover + brush selection are standardized via `ChartLink`.
- Interaction binding is data-driven via `ChartInputBindings`.

## Notable implementation update

- `PolarChartMode::Parallel` now has a dedicated parallel-coordinates render path (axis + polyline), replacing the prior stub.
  - Evidence: `crates/blinc_charts/src/polar.rs` (`render_parallel`).
- Reusable utility layers (`axis`, `scale`, `format`, `time_format`) and optional algorithm layers (`spatial_index`, `triangulation`, `polygon`) were added for gap closure.
- Transition/interpolation baseline now exists via `interpolate.rs` + `transition.rs`, with `GaugeChartModel` as the first integrated chart.

## Split Readiness Judgment (for D3 parity goals)

- If split goal is "Blinc-native interactive chart library with major chart types": ready.
- If split goal is "near D3 module parity": not yet; the main gaps are:
  - Utility/algorithm/transition layers are now present but intentionally minimal.
  - Advanced parity remains open (true delaunay/voronoi, richer scales/axes, broader transition orchestration across charts).

## Minimum actions before split (recommended)

1. Decide parity target explicitly:
   - `Product parity` (recommended): keep current scope, track D3 gaps as backlog.
   - `Library parity`: add utility modules first (`axis`, `scale`, `format`, `time_format`).
2. Complete `BLC-D3-001` ("Parallel coordinates true implementation", replace `render_parallel_stub`) — completed in this branch (initial cut).
3. Complete `BLC-D3-002` (rubric/support policy visibility from README/docs entry) — completed in this branch.
4. Keep this matrix as the contract file for future gap closure checks.
