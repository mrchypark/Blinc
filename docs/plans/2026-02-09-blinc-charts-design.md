# blinc_charts Design (Canvas-First, Ultra-Fast)

Date: 2026-02-09

## Goals

- Canvas-first charts that compose naturally inside Blinc’s layout tree.
- Make overlay / mixing with other canvases easy (crosshair/tooltip layers, annotations, custom UI).
- Use Blinc’s existing interaction/event system (mouse/touch/scroll/pinch/drag) without inventing a parallel event layer.
- Primary target: ultra-fast rendering for huge datasets with minimal CPU/GPU/VRAM overhead.

## Non-Goals (For Now)

- Full D3-like API surface (data join/selection) on day 1.
- Perfect typography/axis/legend system out of the gate.
- Every brush/zoom variant (we’ll add incrementally once the perf foundation is proven).

## Architecture Overview

### 1) Composition Model: `Stack + Canvas Layers`

The core chart element is built as:

- Root: `stack()` (clipped)
- Child 1: plot `canvas(...)` (expensive layer; data rendering)
- Child 2: overlay `canvas(...).foreground()` (cheap layer; crosshair/tooltip/selection)

This pattern intentionally mirrors how users already build Blinc UIs:

- You can add more children to the returned `stack()` to overlay annotations, markers, or even another chart/canvas.
- You can place the chart stack under other stacks (e.g. multiple independent canvases) without special integration.

### 2) State Split: Pure View State vs. Rendering

`LineChartModel` holds:

- Data: `TimeSeriesF32` (sorted x/y, binary-search hover).
- View: `ChartView { domain, padding }` (data<->pixel mapping).
- Interaction state: crosshair x, hover point.
- LOD caches: downsampled points (data coords) + points in local pixel coords.

Events mutate model state; rendering reads model state.

### 3) Interactivity: Use Blinc Events Directly

The chart uses Blinc’s event routing on the root stack:

- `on_mouse_move` -> crosshair + nearest-point hover
- `on_scroll` / `on_pinch` -> zoom X about cursor
- `on_drag` -> pan X

Important: event handlers call `blinc_layout::stateful::request_redraw()` so we redraw without rebuilding the element tree.

### 4) Rendering API: `DrawContext::stroke_polyline`

We standardize large-polyline drawing behind:

`DrawContext::stroke_polyline(points, stroke, brush)`

Default implementation: builds a `Path` and calls `stroke_path`.

GPU implementation: for solid strokes (no dash), pushes compact line segment instances.

This keeps `blinc_charts` simple (always call `stroke_polyline`) while letting the renderer choose the fastest backend.

## Performance Strategy

### 1) LOD: Bounded by Pixels

For time-series lines, rendering “all points” is wasteful beyond screen resolution.

We use bucketed min/max downsampling:

- O(N) per resample
- Output size bounded by plot width (roughly 2 points per pixel column)
- Preserves spikes better than naive uniform decimation

### 2) Caching: Hover Should Be Cheap

Crosshair movement should not trigger resampling.

We cache the resample result keyed by:

- visible X domain (min/max)
- plot size (w/h)

On hover-only redraws, plot reuses cached `points_px` and only redraws the polyline.

### 3) GPU Polyline Fast Path (Low Bandwidth)

Instead of tessellating a path into many vertices, we render each segment as one instance:

- CPU uploads one `GpuLineSegment` per segment: `(x0,y0,x1,y1, clip, color, half_width, z_layer)`
- GPU expands it into a quad in the shader
- Clipping is handled via clip bounds in shader (rect clip)

This keeps memory bandwidth low and makes “very long lines” feasible.

### 4) Safety Limits

Renderer config adds `max_line_segments` (and env override `BLINC_GPU_MAX_LINE_SEGMENTS`)
to avoid runaway buffer growth when a caller accidentally pushes millions of segments.

## Reference: gpui-d3rs (What We’re Borrowing)

We are intentionally aligning with gpui-d3rs’s separation of concerns:

- zoom/brush state as pure data structures
- shape generators separate from interaction state
- renderer/batching isolated behind a small API

But we diverge for perf:

- gpui-d3rs’s GPU2D lines expand segments into per-vertex quads on CPU.
- Blinc’s path is instance-based segments to reduce CPU work + upload volume.

## Roadmap (Next Additions)

1. Multi-series line chart (multiple `TimeSeriesF32` + shared X domain).
2. Y auto-scale for visible range (fast min/max over window, optional).
3. Axes + tick generation (likely minimal scale utilities, then themeable axes).
4. Brush selection (rectangle / X-only) with domain conversion.
5. Area chart (fill) using either:
   - GPU triangles for the band, or
   - SDF/path fallback when needed
6. High-performance picking for dense series:
   - nearest-by-x is enough for time-series; extend to quadtree for scatter later.

