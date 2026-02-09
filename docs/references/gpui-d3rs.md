# gpui-d3rs Reference Notes (for blinc_charts)

Source: `pierreaubert/sotf` -> `crates/gpui-d3rs`

This note is a map of the codebase and the patterns we’re reusing (or intentionally not reusing) in `blinc_charts`.

## What gpui-d3rs Is

- A D3.js-inspired visualization library implemented in Rust on top of GPUI.
- Uses builder-style configs + pure state structs for interaction primitives.

Key modules (top-level `src/`):

- `scale/` (linear/log/ordinal + tick generation)
- `shape/` (line/area/bar/scatter/arc/etc)
- `axis/` (rendering + config/theme)
- `zoom/`, `brush/` (interaction state)
- `gpu2d/` (a GPU primitive renderer: lines/rect/circle/triangle/text)

## Interaction Primitives

### `zoom/mod.rs`

Pattern:

- `ZoomState` stores:
  - “original” x/y domain bounds
  - “current” x/y domain bounds
  - a zoom history stack for reset/back semantics
- `ZoomConfig` provides constraints:
  - per-axis enable
  - min/max extents as fractions of original

Takeaway for Blinc:

- Keep zoom state *pure data* (domain bounds + history), driven by Blinc’s events.
- Constrain zoom at the domain level (avoid view collapsing to tiny spans).

### `brush/mod.rs`

Pattern:

- `BrushState` is a small state machine: start/update/end/reset.
- `BrushSelection` stores a rectangle in pixel coords.
- `to_domain(...)` converts selection to domain coords using scale inversion.
- Explicitly discusses inverted Y ranges (common in charts) and uses `invert()` to do the right thing.

Takeaway for Blinc:

- We should represent brush in pixel coords and convert via domain mapping (or a future `Scale` trait).
- Inversion logic belongs in the mapping layer, not the brush state machine.

## Shapes (Marks)

### `shape/line.rs`

Pattern:

- `LineConfig` is a builder-style config (color/width/opacity/curve/points).
- Data points are normalized to relative coords and then clipped.
- Uses a classic Cohen-Sutherland clipper to drop segments outside the plot box.
- Uses GPUI’s path building for vector-quality strokes.

Takeaway for Blinc:

- Config objects are a good API shape for charts.
- CPU-side clipping is a useful optimization (optional for us because we have shader clip bounds).

### `shape/area.rs` (and `shape/path.rs`)

Pattern:

- “generator” style: closures for x/y accessors + curve handling + path builder output.

Takeaway for Blinc:

- For advanced charts, separate “geometry generation” from “rendering backend”.
- But for the first performance-focused MVP, keep the model minimal (time-series specialized).

## GPU2D Renderer

### `gpu2d/primitives/line.rs`

Pattern:

- CPU expands every segment to 4 vertices + 6 indices (quad per segment).
- Vertex shader then just offsets by the precomputed normal.

Tradeoff:

- Simpler shaders, but high CPU work + high upload bandwidth for huge polylines.

### `gpu2d/renderer.rs` + `gpu2d/shaders.rs`

Pattern:

- Small, explicit batches per primitive type.
- A shared `viewport_size` uniform and a `pixel_to_ndc` helper in WGSL.
- Text is handled via an atlas.

Takeaway for Blinc:

- The “batch then draw once” model is correct for performance.
- For time-series lines, we want *instance data per segment* rather than per-vertex expansion.

## How This Maps to `blinc_charts`

What we adopted:

- Interaction = pure state updated by framework events (Blinc).
- A small view/domain mapping layer separate from rendering.
- Builder-like configs for styling and later API expansion.

What we changed for performance:

- `DrawContext::stroke_polyline` exists to hide backend differences.
- GPU path uses compact `GpuLineSegment` instance data, letting the shader expand to quads.
- LOD/downsample is mandatory for “huge N” time-series, bounded by pixels.

## Suggested Next Borrow

- `ZoomState`-style history and constraints, wired to Blinc pinch/scroll/drag.
- `BrushState` for selection-zoom (X-only and 2D).
- Minimal scale utilities + tick generation (start small; expand as needed).

