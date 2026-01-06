# Layout Animation Investigation Plan

## Problem Statement

Layout animation (`animate_layout()`) is technically working - animations are created, bounds are interpolated, and animated values are returned during render. However, the visual effect is not visible to the user.

## Current Status

### What Works
1. **Animation triggering** - When accordion state changes, `LayoutAnimationState::from_bounds_change()` creates animations
2. **Bounds tracking** - `previous_bounds_by_key` correctly stores old bounds by stable key
3. **Animation persistence** - Animations survive Stateful rebuilds via stable key lookup
4. **Animated values** - Springs are created and interpolating (confirmed via logs showing height going 41 -> 42.08 -> 44.69... -> 109)
5. **Render path** - `get_render_bounds()` returns animated bounds during render

### What Doesn't Work
- Visual animation is not visible despite all the above working

## Investigation Areas

### 1. Children Rendering Inside Animated Container

**Hypothesis**: Children are rendered at their final positions inside the animated container. The container clips them, but the visual effect is "content appears clipped" rather than "content grows smoothly".

**To Investigate**:
- [ ] Check how children bounds are calculated relative to parent
- [ ] Verify if children use parent's animated bounds or layout bounds
- [ ] Test with a simple colored div as content (no text) to see clipping

**Test**: Create a minimal test case:
```rust
// Animated container with solid color child
let mut container = div()
    .w(200.0)
    .overflow_clip()
    .animate_layout(LayoutAnimationConfig::height().with_key("test"))

if expanded {
    container = container.child(
        div().h(100.0).bg(Color::RED)  // Simple solid block
    );
}
```

### 2. Clip Rect Application

**Hypothesis**: The clip rect might not be applied correctly, or it's applied at the wrong coordinates.

**To Investigate**:
- [ ] Add debug logging to clip rect application in `render_layer_with_motion`
- [ ] Verify clip rect uses animated bounds (width/height) not layout bounds
- [ ] Check if `has_layout_animation` returns true during render

**Code Location**: [renderer.rs:4220-4231](crates/blinc_layout/src/renderer.rs#L4220-L4231)
```rust
let clips_content = render_node.props.clips_content || has_layout_animation;
if clips_content {
    let clip_rect = Rect::new(0.0, 0.0, bounds.width, bounds.height);
    // ...
}
```

### 3. Transform Stack and Position

**Hypothesis**: The position transform might be using layout bounds instead of animated bounds, causing the animation to be "offset".

**To Investigate**:
- [ ] Check line 4138: `ctx.push_transform(Transform::translate(bounds.x, bounds.y))`
- [ ] Verify `bounds` comes from `get_render_bounds()` which should return animated values
- [ ] Add logging to confirm animated x/y values during render

### 4. Parent Container Influence

**Hypothesis**: The parent container's layout might be overriding the animated bounds.

**To Investigate**:
- [ ] Check if accordion's parent (`content` div with `flex_col`) affects rendering
- [ ] Test with accordion as the only child (no siblings)
- [ ] Verify `flex_auto()` on item_div isn't causing issues

### 5. Separator Elements

**Hypothesis**: The `hr` separators between accordion items might be interfering with the layout animation.

**To Investigate**:
- [ ] Test accordion without separators
- [ ] Check if separators have their own bounds that aren't animated

### 6. Multiple Animations Interference

**Hypothesis**: Having multiple items with layout animations might cause conflicts.

**To Investigate**:
- [ ] Test with single accordion item only
- [ ] Check if `LayoutAnimationConfig::all()` causes issues (animates x, y, width, height all at once)
- [ ] Try `LayoutAnimationConfig::height()` only for accordion

### 7. Timing and Frame Ordering

**Hypothesis**: Animation might complete before first render, or render happens before animation starts.

**To Investigate**:
- [ ] Add timestamps to animation creation vs first render
- [ ] Check if `update_layout_animations()` runs before or after render
- [ ] Verify animation is still "animating" when render occurs

## Quick Tests to Try

### Test 1: Height-only animation
Change accordion.rs line 355:
```rust
LayoutAnimationConfig::height().with_key(anim_key).gentle()
```

### Test 2: Faster spring
```rust
LayoutAnimationConfig::height().with_key(anim_key).snappy()
```

### Test 3: Single item accordion
Create accordion with only 1 item and no separators.

### Test 4: Compare with working layout_animation_test
The `layout_animation_test()` function in cn_demo.rs should work - verify it does, then identify differences:
- Uses `LayoutAnimationConfig::height()` (not `all()`)
- Uses `snappy()` spring
- Has fixed width (`w(300.0)`)
- No `flex_auto()`

## Debug Logging to Add

```rust
// In get_render_bounds(), when returning animated bounds:
tracing::info!(
    "RENDER: node={:?} key={} animated_bounds={:?} layout_bounds={:?}",
    node_id,
    stable_key,
    current,
    self.layout_tree.get_bounds(node_id, parent_offset)
);

// In render_layer_with_motion, when applying clip:
tracing::info!(
    "CLIP: node={:?} has_layout_anim={} clip_rect={:?}",
    node,
    has_layout_animation,
    clip_rect
);
```

## Architectural Consideration

The current FLIP-style animation animates the **container bounds** while children are laid out at their final positions. This means:

1. Children render at full size inside container
2. Container clips children to animated bounds
3. Visual effect is "reveal" not "grow"

For a true "grow" animation where content appears to expand, we would need either:
- Children to also animate their positions/sizes
- A different approach using transforms (scale from 0 to 1)
- Rendering children at a virtual size that matches animated bounds

## Next Steps

1. Run Test 4 first - verify `layout_animation_test()` works visually
2. If it works, systematically change accordion to match its structure
3. If it doesn't work, the issue is in the core animation system
4. Add debug logging to pinpoint exactly where animated bounds are/aren't being used
