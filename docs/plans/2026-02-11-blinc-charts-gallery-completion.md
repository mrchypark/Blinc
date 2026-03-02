# blinc_charts Gallery Completion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the remaining charts in `crates/blinc_charts` (indices 8–16 in the gallery) as first-class chart modules with Line/Area-level interactivity, and remove placeholders from `charts_gallery_demo`.

**Architecture:** Follow the established `stack() + canvas(plot) + canvas(overlay).foreground()` composition pattern. Each chart module provides `Style`, `Model`, `Handle`, and `*_chart()` APIs, with deterministic GPU-budget caps (max primitives/segments/cells) to prevent renderer buffer overflow.

**Tech Stack:** Rust, `blinc_layout` (events, stack, canvas), `blinc_core` (`DrawContext` primitives), swapchain readback e2e hooks (`BLINC_E2E_*`).

---

## Global Constraints / Guardrails

- **No placeholder UIs** for ITEMS 8–16 in `crates/blinc_app/examples/charts_gallery_demo.rs`.
- **TDD**: For each module, add a minimal failing unit test first (RED), then implement the smallest production code to pass (GREEN), refactor afterwards.
- **No commits** unless explicitly requested (repo agent rule).
- **Performance**: Every new chart must enforce hard budgets (style fields) so drawing can’t exceed default GPU buffers.

## Target: Gallery Items (8–16)

### Item 8: Stacked Area / Streamgraph

**Files:**
- Create: `crates/blinc_charts/src/stacked_area.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs: `Vec<TimeSeriesF32>` with aligned X samples (v1 requirement; reject otherwise).
- Interactions: hover tooltip, wheel/pinch zoom X, drag pan X, Shift+Drag brush X (publish selection).
- Rendering: per-pixel bounded sampling and stacked fills; optional streamgraph baseline mode.

**Test (RED):**
- `StackedAreaChartModel::new(Vec::new())` rejects empty.
- Aligned X validation rejects mismatched lengths or X arrays.

---

### Item 9: Density Map / patch_map (2D)

**Files:**
- Create: `crates/blinc_charts/src/density_map.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs: `Vec<Point>` in data space.
- Interactions: hover tooltip, wheel/pinch zoom both axes about cursor, drag pan (2D), Shift+Drag rectangle brush (selection in data coords).
- Rendering: 2D histogram into bounded grid (max_cells_x/y), rendered as colored rects.

**Test (RED):**
- Empty point list rejects.
- Histogram bins sum == point count (for finite points).

---

### Item 10: Contour / Isobands

**Files:**
- Create: `crates/blinc_charts/src/contour.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs: regular grid `(w, h, values)` similar to heatmap.
- Interactions: same as Density Map (2D pan/zoom + hover).
- Rendering:
  - Isobands: quantized band fill (bounded grid sampling).
  - Contours: marching-squares isolines for N levels (bounded segment budget).

**Test (RED):**
- Marching-squares returns at least 1 segment for a simple plane crossing a threshold.

---

### Item 11: Boxplot / Violin / Error bands

**Files:**
- Create: `crates/blinc_charts/src/statistics.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs: `Vec<Vec<f32>>` groups.
- Interactions: hover tooltip on group, wheel/pinch zoom X (group index domain), drag pan X, Shift brush selection of groups.
- Rendering:
  - Boxplot (median, quartiles, whiskers, optional outliers)
  - Violin (histogram-based KDE approximation)
  - Error bands (optional overlay over a mean line)

**Test (RED):**
- Quantile computation matches known small arrays.

---

### Item 12: Treemap / Sunburst / Icicle / Packing

**Files:**
- Create: `crates/blinc_charts/src/hierarchy.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs: `HierarchyNode { value, label, children }`.
- Interactions: hover tooltip, wheel zoom, drag pan (for treemap/icicle), click to focus (optional v1).
- Rendering:
  - Treemap: squarified layout
  - Icicle: partition layout
  - Sunburst: radial partition (wedge approximation)
  - Packing: naive circle packing (spiral placement)

**Test (RED):**
- Treemap layout keeps all leaf rects within bounds.

---

### Item 13: Graph / Sankey / Chord

**Files:**
- Create: `crates/blinc_charts/src/network.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs:
  - Graph: nodes + edges
  - Sankey: DAG nodes + weighted links
  - Chord: square matrix weights
- Interactions: hover tooltip, wheel zoom, drag pan, click highlight (optional v1).
- Rendering:
  - Graph: deterministic force-layout (capped nodes/edges)
  - Sankey: layered layout + bezier links (capped links)
  - Chord: arcs + ribbon chords (capped ribbons)

**Test (RED):**
- Sankey layer assignment is stable and respects direction (no backward edges within same layer).

---

### Item 14: Parallel / Polar / Radar

**Files:**
- Create: `crates/blinc_charts/src/polar.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs: multi-dim records.
- Interactions: hover + tooltip; zoom/pan (where meaningful).
- Rendering:
  - Parallel coords: polyline per record (capped record count)
  - Radar: polygon per series
  - Polar: radial line/points

**Test (RED):**
- Radar polygon points are finite and within expected radius bounds for normalized inputs.

---

### Item 15: Gauge / Funnel / Streamgraph

**Files:**
- Create: `crates/blinc_charts/src/gauge.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs:
  - Gauge: value + min/max
  - Funnel: stages + values
  - Streamgraph: uses `StackedAreaChartModel` in centered baseline mode
- Interactions: hover tooltip; (streamgraph keeps the stacked-area interactions).

**Test (RED):**
- Gauge clamps value into min..max and maps to angle range deterministically.

---

### Item 16: Geo

**Files:**
- Create: `crates/blinc_charts/src/geo.rs`
- Modify: `crates/blinc_charts/src/lib.rs`
- Modify: `crates/blinc_app/examples/charts_gallery_demo.rs`

**Behavior:**
- Inputs: points/lines/polygons in lon/lat + projection.
- Interactions: wheel zoom, drag pan (2D), hover tooltip.
- Rendering: projected shapes to local pixel space, clipped to plot.

**Test (RED):**
- Projection function maps known lon/lat to finite XY and is monotonic in lon for fixed lat.

---

## Verification

Run:
- `cargo test -p blinc_charts`
- `cargo run -p blinc_app --example charts_gallery_demo --features windowed`
- Optional: `scripts/e2e_macos_charts_gallery.sh` (swapchain readback)

