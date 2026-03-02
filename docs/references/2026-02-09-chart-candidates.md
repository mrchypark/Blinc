## Chart Candidates (D3 / ECharts Reference)

Goal: enumerate likely chart types we will want in `blinc_charts`, then map them to a
Blinc-first implementation plan (canvas-first, overlays, ultra-fast for large data, minimal resources).

This is a candidate list, not a commitment to implement everything.

### Core Families (High Priority)

Time series / numeric
- Line
- Multi-line
- Area / stacked area
- Bar / stacked bar
- Histogram (binning)
- Scatter / bubble
- Candlestick (OHLC)

Density / aggregation (large data)
- Heatmap (2D bin)
- Density map / patch map (existing `patch_map` implementation)
- Contour / isobands (optional)

Analytics
- Boxplot
- Violin
- Error bars / range bands

### Structure & Relationships (Medium Priority)

Networks / flow
- Graph (node-link)
- Sankey
- Chord / ribbon (optional)

Hierarchical
- Treemap
- Sunburst / icicle (partition)
- Tree / dendrogram / cluster
- Circle packing

Coordinates
- Polar charts (radial variants)
- Parallel coordinates

### Specialized & UI Components (Lower Priority / Optional)

Specialized
- Radar
- Gauge
- Funnel
- Theme river / streamgraph variants
- Pictorial bar (if we support symbol fills)

Maps / geo
- Geo map / projections (if Blinc needs it)

Components (not charts, but required for parity)
- Axes + ticks + labels
- Legend
- Tooltip
- Visual mapping (color/size scales)
- Data zoom (x/y)
- Brush selector (x-range, rect, lasso)
- Mark: point/line/area annotations

### Interactions & Linking (Must Have)

We need all 3 selection modes, and they must be sharable across charts.

Selection primitives to standardize:
- X-range selection: `x_min..x_max`
- 2D region selection: rect/lasso in data coords (or converted to an ID set)
- Series selection: set of series IDs

Linking requirements:
- Chart <-> chart domain linking: pan/zoom sync (at least X for time series)
- Selector <-> patch_map: selection must drive patch_map highlighting/aggregation
- Cross-filter: selection in one chart emits a filter event that other charts consume

Proposed shared model (naming TBD):
- `ChartLink`: shared state for `domain`, `hover`, `selection`, `filters`
- `SelectorOverlay`: overlay canvas that owns pointer gestures and updates `ChartLink.selection`

### Implementation Phases (Performance-First)

Phase A (now)
- Stabilize Line / MultiLine (LOD, gaps, pan/zoom)
- Shared selection/link model + brush overlays
- Wire selection to `patch_map` (two-way)

Phase B (fast large-data primitives)
- Scatter: screen-space binning + instanced points
- Bar/histogram: pre-aggregation + instanced rects
- Heatmap/density: 2D binning + patch rendering
- Candlestick: instanced OHLC

Phase C (analysis)
- Boxplot / violin
- Error bars / range bands
- Parallel coordinates

Phase D (structure)
- Treemap / sunburst / icicle
- Graph / sankey (if needed)

### References (Starting Points)

- ECharts series taxonomy: https://echarts.apache.org/
- ECharts custom series examples: https://github.com/apache/echarts-custom-series
- D3 modules:
  - shapes: https://d3js.org/d3-shape
  - hierarchy: https://d3js.org/d3-hierarchy
  - geo: https://d3js.org/d3-geo
  - force: https://d3js.org/d3-force

