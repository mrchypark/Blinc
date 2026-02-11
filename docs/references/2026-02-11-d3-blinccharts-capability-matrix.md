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
- Public chart modules: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/lib.rs`
- Gallery coverage (18 chart kinds wired): `/Users/cypark/Documents/project/Blinc/crates/blinc_app/examples/charts_gallery_demo.rs`
- Shared view/domain mapping: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/view.rs`
- Shared link/selection state: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/link.rs`
- Shared interaction bindings: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/input.rs`

## Capability Matrix (D3 30 modules)

| D3 module | Primary capability | BlincCharts mapping | Status | Evidence / note |
|---|---|---|---|---|
| `d3-array` | stats, bins, extents | histogram/statistics, domain computations | Partial | histogram/statistics exist, but not a general array utility API |
| `d3-axis` | reusable axis generators | per-chart inline grid/labels only | Missing | no dedicated axis module in `blinc_charts/src` |
| `d3-brush` | 1D/2D brushing | `BrushX`, `BrushRect`, linked selection | Full | `brush.rs`, `xy_stack.rs`, `density_map.rs`, `contour.rs` |
| `d3-chord` | chord layout/rendering | `NetworkMode::Chord` | Partial | chord mode exists in `network.rs`, simplified pipeline |
| `d3-color` | color parsing/manipulation | `blinc_core::Color` usage in styles | Partial | chart styles use colors, but no d3-like color utility package |
| `d3-contour` | contour / isobands | `contour.rs` | Full | marching-squares style contour rendering implemented |
| `d3-delaunay` | delaunay/voronoi | none | Missing | no triangulation/voronoi support found |
| `d3-dispatch` | event dispatch bus | framework events + shared state handles | Partial | interactions are event callbacks, no standalone dispatch API |
| `d3-drag` | drag behavior | drag pan/brush in all interactive charts | Full | `on_drag*` patterns in line/bar/scatter/etc |
| `d3-dsv` | CSV/TSV parsing | none in `blinc_charts` | Out-of-scope | data ingestion is outside chart rendering crate |
| `d3-ease` | easing functions | none in `blinc_charts` | Out-of-scope | animation easing belongs to other runtime layers |
| `d3-fetch` | web fetch helpers | none in `blinc_charts` | Out-of-scope | network/data fetch is outside chart crate |
| `d3-force` | force simulation | graph layout mode in `network.rs` | Partial | graph exists, but no standalone configurable force engine API |
| `d3-format` | number formatting | ad-hoc text formatting only | Missing | no shared numeric formatting utility in `blinc_charts` |
| `d3-geo` | geo projections/pathing | `geo.rs` | Partial | geo chart exists; projection set is narrower than d3-geo |
| `d3-hierarchy` | tree/treemap/partition/pack | `hierarchy.rs` | Partial | Treemap/Icicle/Sunburst/Packing exist, narrower API surface |
| `d3-interpolate` | interpolation utilities | ad-hoc chart-level interpolation | Missing | no reusable interpolation module exposed |
| `d3-path` | path serialization | `blinc_core::Path` drawing | Partial | path drawing exists, but not a d3-path-like utility package |
| `d3-polygon` | polygon ops | no general polygon utility API | Missing | chart internals only |
| `d3-quadtree` | spatial index | none | Missing | no quadtree module found |
| `d3-random` | random distributions | demo-local generators only | Out-of-scope | random helpers are in gallery example, not chart API |
| `d3-scale` | scale primitives | `Domain1D/2D` + px/data transforms | Partial | linear mapping exists (`view.rs`), no band/log/time scale module |
| `d3-scale-chromatic` | color palettes | per-chart hardcoded palettes | Partial | colors exist, no palette package surface |
| `d3-selection` | DOM selections | n/a in Blinc UI runtime | Out-of-scope | Blinc uses element builders/events, not DOM selection |
| `d3-shape` | line/area/arc/symbol generators | line/area/bar/scatter/polar etc | Partial | many shape charts exist; no generic shape generator API |
| `d3-time` | time intervals/scales | numeric X domain time-series | Partial | time series support exists (`time_series.rs`), no rich calendar API |
| `d3-time-format` | locale time formatting | none | Missing | no dedicated time formatter in chart crate |
| `d3-timer` | frame timers | none in `blinc_charts` | Out-of-scope | scheduling runtime is outside chart crate |
| `d3-transition` | declarative transitions | none in `blinc_charts` | Missing | no transition API in chart module |
| `d3-zoom` | pan/zoom behavior | scroll/pinch/drag pan across chart types | Full | widespread `on_scroll`/`on_pinch`/`on_drag_pan_total` |

## BlincCharts coverage snapshot

- Implemented chart families (18 kinds wired in gallery): line, multi-line, area, bar, histogram, scatter, candlestick, heatmap, stacked area, density map, contour, statistics, hierarchy, network, polar, gauge, funnel, geo.
- X-domain linking + hover + brush selection are standardized via `ChartLink`.
- Interaction binding is data-driven via `ChartInputBindings`.

## Notable gap found during audit

- `PolarChartMode::Parallel` is currently a stub message (`"parallel (uses radar v1)"`) and reuses radar rendering.
  - Evidence: `/Users/cypark/Documents/project/Blinc/crates/blinc_charts/src/polar.rs` (`render_parallel_stub`).

## Split Readiness Judgment (for D3 parity goals)

- If split goal is "Blinc-native interactive chart library with major chart types": ready.
- If split goal is "near D3 module parity": not yet; the main gaps are:
  - Missing reusable utility layers: axis, scale variants, numeric/time formatting.
  - Missing algorithm modules: delaunay/quadtree/polygon ops.
  - Missing transition/interpolation surface.

## Minimum actions before split (recommended)

1. Decide parity target explicitly:
   - `Product parity` (recommended): keep current scope, track D3 gaps as backlog.
   - `Library parity`: add utility modules first (`axis`, `scale`, `format`, `time_format`).
2. Promote one clear roadmap issue for "Parallel coordinates true implementation" (replace `render_parallel_stub`).
3. Keep this matrix as the contract file for future gap closure checks.
