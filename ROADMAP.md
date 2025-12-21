# Blinc Roadmap: Text, SVG, and Music Player Demo

## Phase 1: High-Quality Font Rendering

### Current State
- **API Ready**: TextStyle, DrawContext.draw_text(), TEXT_SHADER all defined
- **GPU Pipeline Ready**: SDF glyph shader, GpuGlyph struct, atlas texture binding
- **0% Implemented**: No font loading, no glyph rasterization, no text shaping

### Implementation Plan

#### 1.1 Font Loading (Dependency: `ttf-parser`)
- Add `ttf-parser` crate for TTF/OTF parsing
- Create `FontFace` struct to hold parsed font data
- Extract glyph outlines, metrics (ascent, descent, line gap)
- Support font weight variants (Regular, Bold, etc.)

#### 1.2 SDF Glyph Rasterization
- Add `msdfgen` or implement custom SDF rasterization
- Generate multi-channel SDF (MSDF) for sharp corners
- Target 32x32 or 48x48 pixel SDF per glyph
- Store SDF values in glyph atlas texture

#### 1.3 Glyph Atlas Management
- Implement shelf/skyline packing algorithm
- Dynamic atlas allocation (start 1024x1024, grow as needed)
- LRU cache eviction for memory management
- Multi-atlas support for large character sets

#### 1.4 Text Shaping (Dependency: `rustybuzz`)
- Integrate HarfBuzz via `rustybuzz` crate
- Proper glyph positioning with kerning
- Ligature support (fi, fl, etc.)
- OpenType feature support

#### 1.5 Text Layout Engine
- Line breaking algorithm (Unicode UAX #14)
- Word/character wrapping
- Alignment (left, center, right, justify)
- Multi-line text measurement
- Baseline handling

#### 1.6 GPU Integration
- Wire font system to GpuPaintContext.draw_text()
- Batch glyphs into GpuGlyph instances
- Upload atlas texture to GPU
- Render via TEXT_SHADER

### Files to Create/Modify
- `crates/blinc_text/` - New crate for text rendering
- `crates/blinc_gpu/src/paint.rs` - Implement draw_text()
- `crates/blinc_widgets/src/text.rs` - Text widget

---

## Phase 2: Complete SVG Rendering

### Current State
- **Path Primitives Ready**: MoveTo, LineTo, QuadTo, CubicTo, ArcTo, Close
- **Lyon Tessellation**: Fill and stroke tessellation working
- **Missing**: SVG parsing, arc tessellation, viewBox handling

### Implementation Plan

#### 2.1 SVG Parser (Dependency: `usvg` or `roxmltree`)
- Add lightweight SVG parser
- Parse SVG path `d` attribute to blinc PathCommand
- Handle basic SVG elements: path, rect, circle, ellipse, line, polyline, polygon

#### 2.2 Arc Tessellation Fix
- Current: Arcs approximated as straight lines
- Fix: Proper arc-to-bezier conversion in `path_to_lyon_events()`
- Handle large_arc and sweep flags correctly

#### 2.3 SVG Transform Support
- Parse `transform` attribute (translate, rotate, scale, matrix)
- Apply transforms during path conversion
- Handle viewBox coordinate system

#### 2.4 SVG Styling
- Map SVG `fill` to blinc Brush
- Map SVG `stroke` to blinc Stroke
- Support basic CSS colors and gradients
- Handle opacity and fill-rule

#### 2.5 SVG Loader API
```rust
// Proposed API
let svg = SvgDocument::load("icon.svg")?;
ctx.draw_svg(&svg, Point::new(x, y), scale);
```

### Files to Create/Modify
- `crates/blinc_svg/` - New crate for SVG loading
- `crates/blinc_gpu/src/path.rs` - Fix arc tessellation
- `crates/blinc_core/src/draw.rs` - Add SvgDocument type

---

## Phase 3: Music Player Demo with Liquid Glass

### Reference
See `resource/f4014f239023231.Y3JvcCwxMzgwLDEwODAsMjcwLDA.png`

### Visual Elements to Recreate

#### 3.1 Main Player Card
- Liquid glass rounded rectangle (corner_radius ~24px)
- Title text "Liquid Glass" centered
- Progress bar with scrubber
- Time labels (0:10, -3:24)
- Edge highlight with light reflection

#### 3.2 Playback Controls
- Rewind icon (double chevron left)
- Pause button (two vertical bars)
- Fast-forward icon (double chevron right)
- White fill, centered in glass panel

#### 3.3 Volume/AirPlay Section
- Volume bars indicator (5 bars, varying height)
- AirPlay circular button with concentric circles icon

#### 3.4 Bottom Toolbar
- Two circular glass buttons
- Flashlight icon (left)
- Camera icon (right)
- Smaller than main panel, pill-shaped

#### 3.5 Background
- Blurred nature/leaf image
- Green/olive color palette

### Implementation in Test Suite

```rust
// In crates/blinc_test_suite/src/tests/glass.rs
suite.add_glass("music_player_demo", |ctx| {
    let c = ctx.ctx();

    // Background gradient (simulating blurred image)
    // Main glass panel
    // Title text
    // Progress bar
    // Playback control icons (paths)
    // Volume indicator
    // Bottom toolbar buttons
});
```

### Required Features
1. **Text rendering** (Phase 1) - For "Liquid Glass", time labels
2. **SVG/Path rendering** (Phase 2) - For icons (play, pause, etc.)
3. **Liquid glass** (Done) - For panels and buttons
4. **Rounded rectangles** (Done) - For progress bar, buttons

### Milestone Checkpoints
- [ ] Render main glass panel with correct proportions
- [ ] Add progress bar UI element
- [ ] Render playback icons using paths
- [ ] Add text labels (requires Phase 1)
- [ ] Add volume indicator bars
- [ ] Add bottom toolbar with circular glass buttons
- [ ] Final polish: shadows, exact colors, spacing

---

## Dependencies to Add

```toml
# Cargo.toml workspace dependencies
ttf-parser = "0.21"      # Font parsing
rustybuzz = "0.14"       # Text shaping (HarfBuzz)
msdfgen = "0.2"          # SDF glyph generation (optional)
usvg = "0.42"            # SVG parsing (or roxmltree for lightweight)
```

---

## Timeline Priority

1. **Immediate**: Music player demo WITHOUT text (shapes, icons as paths, glass)
2. **Short-term**: Basic text rendering (single font, ASCII)
3. **Medium-term**: Full text stack (shaping, multi-font, Unicode)
4. **Long-term**: Complete SVG support with all features

This allows incremental progress with visible results at each stage.
