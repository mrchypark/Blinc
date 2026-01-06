# Layout Animation System - Design Plan

## Overview

A declarative system for animating layout changes (position, size) automatically when element bounds change. Unlike `motion()` containers which use visual transforms, this system animates the **rendered bounds** while letting layout settle instantly.

## Problem Statement

Currently, when an accordion section opens:
1. State changes → triggers tree rebuild
2. Taffy computes new layout (content expanded)
3. Rendering uses new bounds immediately (jarring jump)

**Goal**: Smooth visual interpolation from old bounds to new bounds, while layout settles instantly (siblings position correctly).

## Architecture: FLIP-Style Animation

**FLIP** = First, Last, Invert, Play

1. **First**: Capture old bounds before layout change
2. **Last**: Compute new layout (taffy runs)
3. **Invert**: Calculate delta (visual offset to show old position)
4. **Play**: Animate delta back to zero (smooth transition to new position)

## Key Design Decisions

### 1. Layout Happens Instantly, Animation is Visual

- Taffy computes final layout immediately
- Siblings get correct positions without waiting
- Animating element **renders** at interpolated bounds
- Content clips to animated bounds during transition

### 2. Leverage Existing Infrastructure

- **AnimatedValue** for spring physics
- **Global scheduler** for animation ticking
- **MotionBindings pattern** for frame-perfect sampling
- **on_layout callback** to detect bounds changes

### 3. Opt-in Per Element

Not all elements should animate layout changes. New API:

```rust
div()
    .animate_layout(LayoutAnimation::height())  // Animate height changes
    .child(content)
```

## Implementation Plan

### Phase 1: Core Data Structures

#### 1.1 LayoutAnimationConfig

```rust
// crates/blinc_layout/src/layout_animation.rs

/// Configuration for which properties to animate
#[derive(Clone, Debug)]
pub struct LayoutAnimationConfig {
    /// Animate height changes
    pub height: bool,
    /// Animate width changes
    pub width: bool,
    /// Animate x position changes
    pub x: bool,
    /// Animate y position changes
    pub y: bool,
    /// Spring configuration
    pub spring: SpringConfig,
    /// Minimum change threshold (ignore tiny changes)
    pub threshold: f32,
}

impl LayoutAnimationConfig {
    pub fn height() -> Self {
        Self {
            height: true,
            width: false,
            x: false,
            y: false,
            spring: SpringConfig::snappy(),
            threshold: 1.0,
        }
    }

    pub fn all() -> Self {
        Self {
            height: true,
            width: true,
            x: true,
            y: true,
            spring: SpringConfig::snappy(),
            threshold: 1.0,
        }
    }

    pub fn with_spring(mut self, spring: SpringConfig) -> Self {
        self.spring = spring;
        self
    }
}
```

#### 1.2 LayoutAnimationState

```rust
/// Active animation state for a single element
pub struct LayoutAnimationState {
    /// Starting bounds (captured before layout change)
    pub start_bounds: ElementBounds,
    /// Target bounds (from taffy layout)
    pub end_bounds: ElementBounds,
    /// Animated values for each property
    pub height_anim: Option<AnimatedValue>,
    pub width_anim: Option<AnimatedValue>,
    pub x_anim: Option<AnimatedValue>,
    pub y_anim: Option<AnimatedValue>,
}

impl LayoutAnimationState {
    /// Get current interpolated bounds
    pub fn current_bounds(&self) -> ElementBounds {
        ElementBounds {
            x: self.x_anim.as_ref().map(|a| a.get()).unwrap_or(self.end_bounds.x),
            y: self.y_anim.as_ref().map(|a| a.get()).unwrap_or(self.end_bounds.y),
            width: self.width_anim.as_ref().map(|a| a.get()).unwrap_or(self.end_bounds.width),
            height: self.height_anim.as_ref().map(|a| a.get()).unwrap_or(self.end_bounds.height),
        }
    }

    /// Check if any animations are still running
    pub fn is_animating(&self) -> bool {
        self.height_anim.as_ref().map(|a| a.is_animating()).unwrap_or(false)
            || self.width_anim.as_ref().map(|a| a.is_animating()).unwrap_or(false)
            || self.x_anim.as_ref().map(|a| a.is_animating()).unwrap_or(false)
            || self.y_anim.as_ref().map(|a| a.is_animating()).unwrap_or(false)
    }
}
```

### Phase 2: RenderTree Integration

#### 2.1 Storage in RenderTree

```rust
// In renderer.rs - RenderTree struct

pub struct RenderTree {
    // ... existing fields ...

    /// Layout animation configs (opt-in per element)
    layout_animation_configs: HashMap<LayoutNodeId, LayoutAnimationConfig>,

    /// Active layout animations (created when bounds change)
    layout_animations: HashMap<LayoutNodeId, LayoutAnimationState>,

    /// Previous frame bounds (for detecting changes)
    previous_bounds: HashMap<LayoutNodeId, ElementBounds>,
}
```

#### 2.2 Collection During Build

```rust
// In build_element / rebuild_element

fn build_element(&mut self, element: &dyn ElementBuilder) -> LayoutNodeId {
    let node_id = /* ... existing build logic ... */;

    // Collect layout animation config if element has one
    if let Some(config) = element.layout_animation_config() {
        self.layout_animation_configs.insert(node_id, config);
    }

    node_id
}
```

#### 2.3 ElementBuilder Trait Extension

```rust
// In div.rs - ElementBuilder trait

pub trait ElementBuilder: Send + Sync {
    // ... existing methods ...

    /// Get layout animation configuration (if any)
    fn layout_animation_config(&self) -> Option<LayoutAnimationConfig> {
        None
    }
}
```

### Phase 3: Animation Triggering

#### 3.1 Detect Bounds Changes After Layout

```rust
// In renderer.rs - after compute_layout()

impl RenderTree {
    pub fn compute_layout(&mut self, width: f32, height: f32) {
        // Run taffy layout
        self.layout_tree.compute_layout(/* ... */);

        // Check for layout animations AFTER layout is computed
        self.update_layout_animations();

        // ... existing post-layout work ...
    }

    fn update_layout_animations(&mut self) {
        let scheduler = get_global_scheduler()
            .expect("Scheduler not initialized");

        for (&node_id, config) in &self.layout_animation_configs {
            // Get new bounds from taffy
            let Some(new_bounds) = self.layout_tree.get_bounds(node_id, (0.0, 0.0)) else {
                continue;
            };

            // Get previous bounds (or use new bounds if first frame)
            let old_bounds = self.previous_bounds
                .get(&node_id)
                .copied()
                .unwrap_or(new_bounds);

            // Check if bounds changed significantly
            let height_changed = config.height
                && (new_bounds.height - old_bounds.height).abs() > config.threshold;
            let width_changed = config.width
                && (new_bounds.width - old_bounds.width).abs() > config.threshold;
            let x_changed = config.x
                && (new_bounds.x - old_bounds.x).abs() > config.threshold;
            let y_changed = config.y
                && (new_bounds.y - old_bounds.y).abs() > config.threshold;

            if height_changed || width_changed || x_changed || y_changed {
                // Create or update animation state
                let state = LayoutAnimationState {
                    start_bounds: old_bounds,
                    end_bounds: new_bounds,
                    height_anim: if height_changed {
                        let mut anim = AnimatedValue::new(
                            scheduler.clone(),
                            old_bounds.height,
                            config.spring,
                        );
                        anim.set_target(new_bounds.height);
                        Some(anim)
                    } else { None },
                    width_anim: if width_changed {
                        let mut anim = AnimatedValue::new(
                            scheduler.clone(),
                            old_bounds.width,
                            config.spring,
                        );
                        anim.set_target(new_bounds.width);
                        Some(anim)
                    } else { None },
                    x_anim: if x_changed {
                        let mut anim = AnimatedValue::new(
                            scheduler.clone(),
                            old_bounds.x,
                            config.spring,
                        );
                        anim.set_target(new_bounds.x);
                        Some(anim)
                    } else { None },
                    y_anim: if y_changed {
                        let mut anim = AnimatedValue::new(
                            scheduler.clone(),
                            old_bounds.y,
                            config.spring,
                        );
                        anim.set_target(new_bounds.y);
                        Some(anim)
                    } else { None },
                };

                self.layout_animations.insert(node_id, state);
            }

            // Store current bounds for next frame comparison
            self.previous_bounds.insert(node_id, new_bounds);
        }

        // Clean up completed animations
        self.layout_animations.retain(|_, state| state.is_animating());
    }
}
```

### Phase 4: Rendering with Animated Bounds

#### 4.1 Get Animated Bounds During Render

```rust
// In renderer.rs - render methods

impl RenderTree {
    /// Get bounds for rendering, using animated values if animation is active
    pub fn get_render_bounds(&self, node_id: LayoutNodeId, parent_offset: (f32, f32)) -> Option<ElementBounds> {
        // First get taffy layout bounds
        let layout_bounds = self.layout_tree.get_bounds(node_id, parent_offset)?;

        // Check if we have an active animation for this node
        if let Some(anim_state) = self.layout_animations.get(&node_id) {
            // Return interpolated bounds
            let animated = anim_state.current_bounds();
            Some(ElementBounds {
                x: layout_bounds.x, // Use layout position (siblings depend on it)
                y: layout_bounds.y,
                width: animated.width,  // Use animated size for visual
                height: animated.height,
            })
        } else {
            Some(layout_bounds)
        }
    }
}
```

#### 4.2 Modify render_node to Use Animated Bounds

```rust
// In render_node / paint_node

fn render_node(&self, ctx: &mut dyn DrawContext, node: LayoutNodeId, parent_offset: (f32, f32)) {
    // Use animated bounds instead of raw layout bounds
    let bounds = match self.get_render_bounds(node, parent_offset) {
        Some(b) => b,
        None => return,
    };

    // Check if we're animating (need to clip content)
    let is_animating = self.layout_animations.contains_key(&node);

    if is_animating {
        // Push clip rect to animated bounds
        ctx.push_clip(Rect::new(
            bounds.x + parent_offset.0,
            bounds.y + parent_offset.1,
            bounds.width,
            bounds.height,
        ));
    }

    // ... rest of rendering ...

    if is_animating {
        ctx.pop_clip();
    }
}
```

### Phase 5: API for Div

#### 5.1 Div Builder Method

```rust
// In div.rs

impl Div {
    /// Animate layout changes for this element
    ///
    /// When bounds change (due to content changes, state changes, etc.),
    /// the element will smoothly animate from old bounds to new bounds.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Animate height changes (good for accordions, collapsibles)
    /// div()
    ///     .animate_layout(LayoutAnimation::height())
    ///     .child(collapsible_content)
    ///
    /// // Animate all bound changes with custom spring
    /// div()
    ///     .animate_layout(
    ///         LayoutAnimation::all()
    ///             .with_spring(SpringConfig::wobbly())
    ///     )
    ///     .child(content)
    /// ```
    pub fn animate_layout(mut self, config: LayoutAnimationConfig) -> Self {
        self.layout_animation_config = Some(config);
        self
    }
}

// Add field to Div struct
pub struct Div {
    // ... existing fields ...
    layout_animation_config: Option<LayoutAnimationConfig>,
}

// Implement in ElementBuilder
impl ElementBuilder for Div {
    fn layout_animation_config(&self) -> Option<LayoutAnimationConfig> {
        self.layout_animation_config.clone()
    }
}
```

### Phase 6: Accordion Integration

#### 6.1 Update Accordion to Use Layout Animation

```rust
// In accordion.rs

// The item container that wraps trigger + content
let item_div = div()
    .animate_layout(LayoutAnimationConfig::height())  // Animate height!
    .padding_x(Length::Px(12.0))
    .flex_col()
    .flex_auto()
    .overflow_clip()
    .w_full()
    .child(trigger)
    .child(collapsible_content);
```

Now accordion sections will smoothly animate height when opening/closing!

## Implementation Order

1. **Phase 1**: Create `layout_animation.rs` with data structures
2. **Phase 2**: Add storage to RenderTree, extend ElementBuilder trait
3. **Phase 3**: Implement animation triggering in `compute_layout`
4. **Phase 4**: Modify rendering to use animated bounds
5. **Phase 5**: Add `animate_layout()` API to Div
6. **Phase 6**: Update Accordion component

## Testing Strategy

1. **Unit tests**: LayoutAnimationConfig builder patterns
2. **Visual test**: Accordion with animated height
3. **Edge cases**:
   - Rapid open/close (animation interruption)
   - Multiple animated elements
   - Nested layout animations
   - Window resize during animation

## Future Enhancements

1. **Stagger support**: Animate children sequentially
2. **Custom easing**: Support keyframe-based layout animations
3. **Exit animations**: Animate to zero height before removal
4. **Shared element transitions**: Animate position between different parents (hero animations)

## Comparison: Layout Animation vs Motion Container

| Feature | Layout Animation | Motion Container |
|---------|------------------|------------------|
| What animates | Actual rendered bounds | Visual transforms |
| Siblings reflow | Instantly (layout settled) | No (just visual) |
| GPU accelerated | No (bounds recalculated) | Yes (transform matrix) |
| Use case | Accordion, expandable | Fade, slide, scale effects |
| Content clipping | Automatic during animation | Via overflow_clip() |
| Performance | Good (per-element) | Better (GPU transform) |

## Summary

This system provides **declarative layout animations** that:
- Integrate with existing animation scheduler
- Use proven spring physics
- Require minimal API surface (`.animate_layout()`)
- Handle accordion/collapsible use cases elegantly
- Maintain layout integrity (siblings position correctly)
