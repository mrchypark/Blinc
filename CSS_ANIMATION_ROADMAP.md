# CSS Animation Properties Roadmap

Tracks remaining standard CSS animatable properties, ordered by impact.

**Current coverage**: 49 properties animatable via `@keyframes` + `transition:`

---

## Phase 1: Easy Wins — Already Parsed, Just Need Animation Wiring

These exist in `ElementStyle` and are parsed by the CSS engine. They only need:
1. `KeyframeProperties` field
2. `lerp()` interpolation
3. `style_to_keyframe_properties()` extraction
4. `apply_keyframe_props_to_render()` application
5. `snapshot_keyframe_properties()` capture
6. `check_transition!` entry in `detect_and_start_transitions()`

### Layout Constraints
- [x] `min-width` — `Option<f32>`
- [x] `max-width` — `Option<f32>`
- [x] `min-height` — `Option<f32>`
- [x] `max-height` — `Option<f32>`

### Positioning
- [x] `top` — `Option<f32>`
- [x] `right` — `Option<f32>`
- [x] `bottom` — `Option<f32>`
- [x] `left` — `Option<f32>`

### Flex
- [x] `flex-grow` — `Option<f32>`
- [x] `flex-shrink` — `Option<f32>`

### Stacking
- [x] `z-index` — `Option<f32>` (integer in CSS, but f32 for smooth interpolation, round on apply)

**Files**: `keyframe.rs`, `css_parser.rs`, `renderer.rs`
**Estimate**: Mechanical — same pattern as existing layout properties.

---

## Phase 2: Text Color Animation

Highest-impact missing property. Requires threading color through the text rendering pipeline.

- [x] `color` (text foreground color) — `Option<[f32; 4]>`

### Implementation
1. Add `text_color` to `KeyframeProperties`
2. Add `text_color` to `RenderProps` (or use existing if present)
3. In `render_layer_with_motion`, pass animated color to text draw calls
4. Wire through `snapshot_keyframe_properties` / `apply_keyframe_props_to_render`
5. Add `check_transition!(text_color, "color")` with `default [0.0, 0.0, 0.0, 1.0]`

**Files**: `keyframe.rs`, `css_parser.rs`, `renderer.rs`, `element_style.rs`
**Complexity**: Medium — need to trace how text color flows through `draw_text()` / `GlyphInstance`.

---

## Phase 3: Transform Extensions

### Skew
- [x] `skewX` — `Option<f32>` (degrees)
- [x] `skewY` — `Option<f32>` (degrees)

### Implementation
1. Add `skew_x`, `skew_y` to `KeyframeProperties`
2. Parse `skew()`, `skewX()`, `skewY()` in transform parser
3. Add skew to `Affine2D` composition in `render_layer_with_motion`
4. GPU: skew can be composed into the existing affine matrix, no shader changes needed

### Transform Origin
- [x] `transform-origin` — `Option<[f32; 2]>` (x%, y%)

### Implementation
1. Store transform-origin on `RenderProps`
2. In `render_layer_with_motion`, offset transform center before applying rotation/scale
3. Animatable via interpolation of origin coordinates

**Files**: `keyframe.rs`, `css_parser.rs`, `renderer.rs`, `element_style.rs`

---

## Phase 4: Font Size Animation

- [x] `font-size` — `Option<f32>` (px)

### Implementation
1. Add `font_size` to `KeyframeProperties`
2. In `render_layer_with_motion`, override text style font size from animated value
3. Font size changes require text re-layout (glyph positions change)
4. Need to mark text layout dirty when font-size animates

**Files**: `keyframe.rs`, `css_parser.rs`, `renderer.rs`, text subsystem
**Complexity**: Medium-High — text re-shaping per frame is expensive. May need caching strategy.

---

## Phase 5: Outline Properties

Not currently parsed — need parser + animation support.

- [ ] `outline-color` — `Option<[f32; 4]>`
- [ ] `outline-width` — `Option<f32>`
- [ ] `outline-offset` — `Option<f32>`

### Implementation
1. Add `outline` fields to `ElementStyle` and `RenderProps`
2. Parse `outline`, `outline-color`, `outline-width`, `outline-offset`
3. Render outline as a second SDF rect (bounds expanded by offset+width, no fill, border only)
4. Add to `KeyframeProperties` + animation wiring

**Files**: `element_style.rs`, `css_parser.rs`, `renderer.rs`, `keyframe.rs`
**Complexity**: Medium — new visual feature, but reuses existing SDF rect pipeline.

---

## Phase 6: Gradient Animation

- [ ] Background gradient color stop interpolation

### Implementation
1. Extend `KeyframeProperties` with gradient-specific fields:
   - `gradient_start_color: Option<[f32; 4]>`
   - `gradient_end_color: Option<[f32; 4]>`
   - `gradient_angle: Option<f32>` (for linear)
2. In `apply_keyframe_props_to_render`, reconstruct `Brush::LinearGradient` from interpolated values
3. Parser: extract gradient colors from `Brush` in `style_to_keyframe_properties`
4. Limitation: only 2-stop gradients initially (start + end color)

**Files**: `keyframe.rs`, `css_parser.rs`, `renderer.rs`
**Complexity**: Medium — gradient brush reconstruction from interpolated components.

---

## Phase 7: Text Shadow

- [ ] `text-shadow` — offset, blur, color

### Implementation
1. Add `TextShadow` struct to `element_style.rs`
2. Parse `text-shadow: <offset-x> <offset-y> <blur> <color>`
3. Add `text_shadow_params: Option<[f32; 4]>`, `text_shadow_color: Option<[f32; 4]>` to `KeyframeProperties`
4. In text rendering, draw shadow glyphs offset + blurred before main glyphs

**Files**: `element_style.rs`, `css_parser.rs`, `keyframe.rs`, `renderer.rs`, text shader
**Complexity**: Medium — need shadow pass in text rendering.

---

## Phase 8: Filter Extensions

### blur()
- [ ] `filter: blur(Npx)` — requires multi-pass rendering

### Implementation
1. Render element to offscreen texture
2. Apply Gaussian blur (separable: horizontal + vertical passes)
3. Composite blurred result back
4. Add `filter_blur: Option<f32>` to `KeyframeProperties`
5. GPU: new blur compute/render pass with ping-pong buffers

**Complexity**: High — multi-pass rendering, offscreen textures, significant GPU pipeline work.

### drop-shadow()
- [ ] `filter: drop-shadow(x y blur color)` — similar to blur but shaped

### Implementation
1. Render element alpha to offscreen texture
2. Blur the alpha
3. Colorize and composite behind the original element
4. Reuses blur infrastructure from `blur()`

**Complexity**: High — depends on blur() infrastructure.

---

## Phase 9: Backdrop Filter Animation

- [ ] `backdrop-filter` transition between states (e.g. `blur(0) → blur(20px)`)

Currently parsed as discrete materials (Glass, Chrome, etc.). Animating requires:
1. Parameterize backdrop-filter with continuous values (blur radius, saturation, etc.)
2. Multi-pass: capture background, apply filter, composite element on top
3. Similar infrastructure to `filter: blur()`

**Complexity**: Very High — overlaps with Phase 8 blur infrastructure.

---

## Phase 10: Advanced Selectors

Not animation properties per se, but needed for standard CSS patterns:

- [ ] `+` adjacent sibling combinator
- [ ] `~` general sibling combinator
- [ ] `:not()` pseudo-class
- [ ] `:is()` / `:where()` pseudo-classes
- [ ] `:nth-last-child()`, `:nth-of-type()`, `:first-of-type()`, `:last-of-type()`
- [ ] `:empty`, `:root`
- [ ] `*` universal selector
- [ ] Multiple classes `.class1.class2` (currently single class only?)

**Files**: `css_parser.rs`, `renderer.rs`, `selector/registry.rs`

---

## Summary

| Phase | Properties | Impact | Effort |
|-------|-----------|--------|--------|
| 1 | min/max w/h, top/right/bottom/left, flex-grow/shrink, z-index | High | Low |
| 2 | color (text) | High | Medium |
| 3 | skew, transform-origin | Medium | Medium |
| 4 | font-size | Medium | Medium-High |
| 5 | outline-* | Low-Medium | Medium |
| 6 | gradient interpolation | Medium | Medium |
| 7 | text-shadow | Low-Medium | Medium |
| 8 | filter: blur(), drop-shadow() | High | High |
| 9 | backdrop-filter animation | Medium | Very High |
| 10 | Advanced selectors | Medium | Medium |

**Priority order**: 1 → 2 → 3 → 8 → 6 → 4 → 10 → 5 → 7 → 9
