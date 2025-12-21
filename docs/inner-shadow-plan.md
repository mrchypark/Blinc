# Inner Shadow Implementation Plan

## Problem Statement

The current shadow implementation masks out the shape area so shadows only render *outside* the shape boundary. This is correct for drop shadows but breaks inner shadow effects which need to render *inside* the shape.

## Current Implementation

```
PRIM_SHADOW:
  1. Calculate Gaussian blur shadow at offset position
  2. Mask out shape interior using SDF (shape_mask = smoothstep(shape_d))
  3. Return shadow only where shape_d > 0 (outside shape)
```

## Proposed Solution: Add PRIM_INNER_SHADOW Type

### Approach 1: Inverted SDF Mask

Add a new primitive type `PRIM_INNER_SHADOW` that uses an inverted mask:

```wgsl
case PRIM_INNER_SHADOW: {
    // Inner shadow - only render INSIDE the shape
    let shape_d = sd_rounded_rect(p, origin, size, prim.corner_radius);
    let aa_width = fwidth(shape_d) * 0.5;

    // Invert mask: 1 inside shape, 0 outside
    let shape_mask = 1.0 - smoothstep(-aa_width, aa_width, shape_d);

    // Shadow from the edge inward (negate offset to point inward)
    let inner_offset = -prim.shadow.xy;
    let blur = prim.shadow.z;

    // Calculate shadow distance from edge
    // Inner shadow intensity increases toward the offset direction
    let shadow_d = sd_rounded_rect(p - inner_offset, origin, size, prim.corner_radius);
    let inner_shadow_alpha = 1.0 - smoothstep(0.0, blur, -shadow_d);

    result = prim.shadow_color;
    result.a *= inner_shadow_alpha * shape_mask;
    result.a *= clip_alpha;
    return result;
}
```

### Approach 2: Edge Distance Blur

Calculate shadow intensity based on distance from shape edge:

```wgsl
case PRIM_INNER_SHADOW: {
    let shape_d = sd_rounded_rect(p, origin, size, prim.corner_radius);

    // Only render inside shape (d < 0)
    if shape_d > 0.0 {
        discard;
    }

    let blur = prim.shadow.z;
    let offset = prim.shadow.xy;

    // Distance from edge, accounting for offset direction
    let offset_dir = normalize(offset);
    let offset_influence = dot(offset_dir, normalize(p - center));
    let edge_dist = -shape_d; // positive inside

    // Shadow is stronger near edge in offset direction
    let shadow_intensity = (1.0 - smoothstep(0.0, blur, edge_dist))
                         * (0.5 + 0.5 * offset_influence);

    result = prim.shadow_color;
    result.a *= shadow_intensity;
    return result;
}
```

### Approach 3: Shrunk Shape Subtraction

Create inner shadow by subtracting a slightly shrunk version of the shape:

```wgsl
case PRIM_INNER_SHADOW: {
    let shape_d = sd_rounded_rect(p, origin, size, prim.corner_radius);
    let aa_width = fwidth(shape_d) * 0.5;

    // Only inside shape
    let inside_mask = 1.0 - smoothstep(-aa_width, aa_width, shape_d);
    if inside_mask < 0.001 {
        discard;
    }

    let blur = prim.shadow.z;
    let spread = prim.shadow.w;
    let offset = prim.shadow.xy;

    // Shrunk and offset shape
    let inner_origin = origin + offset + vec2(spread);
    let inner_size = size - vec2(spread * 2.0);
    let inner_d = sd_rounded_rect(p, inner_origin, inner_size, prim.corner_radius);

    // Shadow in the gap between outer and inner shapes
    let shadow_alpha = smoothstep(-blur, blur, inner_d);

    result = prim.shadow_color;
    result.a *= shadow_alpha * inside_mask;
    result.a *= clip_alpha;
    return result;
}
```

## Recommended Approach: Approach 3 (Shrunk Shape Subtraction)

**Rationale:**
1. Most intuitive - matches CSS `box-shadow: inset`
2. Spread parameter works naturally (positive = larger shadow area)
3. Offset moves the shadow source, creating directional effect
4. Blur softens the edge transition
5. No special cases for corner handling

## API Changes

### DrawContext Trait

```rust
/// Draw an inner shadow (inset shadow) for a shape
fn draw_inner_shadow(&mut self, rect: Rect, corner_radius: CornerRadius, shadow: Shadow);
```

### PrimitiveType Enum

```rust
#[repr(u32)]
pub enum PrimitiveType {
    Rect = 0,
    Circle = 1,
    Ellipse = 2,
    Shadow = 3,
    InnerShadow = 4,  // NEW
}
```

### Shader Constants

```wgsl
const PRIM_INNER_SHADOW: u32 = 4u;
```

## Implementation Steps

1. Add `PRIM_INNER_SHADOW` constant to shader
2. Add `InnerShadow` variant to `PrimitiveType` enum
3. Implement inner shadow case in fragment shader (Approach 3)
4. Add `draw_inner_shadow` method to `DrawContext` trait
5. Implement `draw_inner_shadow` in `GpuPaintContext`
6. Add visual tests for inner shadow effects
7. Update `shadow_inner_effect` test to use new API

## Visual Test Cases

```rust
// Basic inner shadow
suite.add("inner_shadow_basic", |ctx| {
    let c = ctx.ctx();
    c.fill_rect(rect, radius, Color::WHITE.into());
    c.draw_inner_shadow(rect, radius, Shadow::new(2.0, 2.0, 8.0, Color::BLACK.with_alpha(0.3)));
});

// Pressed button effect
suite.add("inner_shadow_button", |ctx| {
    let c = ctx.ctx();
    // Outer button with inner shadow for pressed state
    c.fill_rect(rect, radius, Color::rgba(0.9, 0.9, 0.9, 1.0).into());
    c.draw_inner_shadow(rect, radius, Shadow {
        offset_x: 0.0,
        offset_y: 2.0,
        blur: 4.0,
        spread: 0.0,
        color: Color::BLACK.with_alpha(0.2),
    });
});

// Text input field
suite.add("inner_shadow_input", |ctx| {
    let c = ctx.ctx();
    c.fill_rect(rect, 4.0.into(), Color::WHITE.into());
    c.draw_inner_shadow(rect, 4.0.into(), Shadow::new(0.0, 1.0, 3.0, Color::BLACK.with_alpha(0.1)));
    c.stroke_rect(rect, 4.0.into(), &Stroke::new(1.0), Color::rgba(0.8, 0.8, 0.8, 1.0).into());
});
```

## Considerations

### Rendering Order
Inner shadows should be drawn AFTER the shape fill, as an overlay. The shape acts as a natural clip.

### Composition with Drop Shadows
A complete card effect might use both:
```rust
c.draw_shadow(rect, radius, drop_shadow);      // Outer glow
c.fill_rect(rect, radius, color);               // Shape
c.draw_inner_shadow(rect, radius, inner_shadow); // Inner depth
```

### Performance
Single additional primitive per inner shadow, similar cost to drop shadows.

### Limitations
- Complex shapes (paths) may need tessellation-based approach
- Multiple inner shadows stack with alpha blending
- Very large blur values may show artifacts at corners
