//! Motion container for animations
//!
//! A container that applies animations to its children. Supports:
//! - Enter/exit animations (fade_in, scale_in, slide_in, etc.)
//! - Staggered animations for lists
//! - **Continuous animations** driven by `AnimatedValue` or `AnimatedTimeline`
//!
//! # Example - Enter/Exit
//!
//! ```ignore
//! use blinc_layout::prelude::*;
//!
//! motion()
//!     .fade_in(300)
//!     .fade_out(200)
//!     .child(my_content)
//! ```
//!
//! # Example - Continuous Animation with AnimatedValue
//!
//! ```ignore
//! use blinc_layout::prelude::*;
//! use blinc_animation::{AnimatedValue, SpringConfig};
//!
//! // Create animated value for Y translation
//! let offset_y = Rc::new(RefCell::new(
//!     AnimatedValue::new(ctx.animation_handle(), 0.0, SpringConfig::wobbly())
//! ));
//!
//! motion()
//!     .translate_y(offset_y.clone())  // Bind to AnimatedValue
//!     .child(my_content)
//!
//! // Later, in drag handler:
//! offset_y.borrow_mut().set_target(100.0);  // Animates smoothly
//! ```

use crate::div::{ElementBuilder, ElementTypeId};
use crate::element::{MotionAnimation, MotionKeyframe, RenderProps};
use crate::tree::{LayoutNodeId, LayoutTree};
use blinc_animation::{AnimatedValue, AnimationPreset, MultiKeyframeAnimation};
use blinc_core::Transform;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::{Display, FlexDirection, Style};

/// Animation configuration for element lifecycle
#[derive(Clone)]
pub struct ElementAnimation {
    /// The animation to play
    pub animation: MultiKeyframeAnimation,
}

impl ElementAnimation {
    /// Create a new element animation
    pub fn new(animation: MultiKeyframeAnimation) -> Self {
        Self { animation }
    }

    /// Set delay before animation starts
    pub fn with_delay(mut self, delay_ms: u32) -> Self {
        self.animation = self.animation.delay(delay_ms);
        self
    }
}

impl From<MultiKeyframeAnimation> for ElementAnimation {
    fn from(animation: MultiKeyframeAnimation) -> Self {
        Self::new(animation)
    }
}

/// Direction for stagger animations
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StaggerDirection {
    /// Animate first to last
    #[default]
    Forward,
    /// Animate last to first
    Reverse,
    /// Animate from center outward
    FromCenter,
}

/// Direction for slide animations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlideDirection {
    Left,
    Right,
    Top,
    Bottom,
}

/// Configuration for stagger animations
#[derive(Clone)]
pub struct StaggerConfig {
    /// Delay between each child's animation start (ms)
    pub delay_ms: u32,
    /// Animation to apply to each child
    pub animation: ElementAnimation,
    /// Direction of stagger
    pub direction: StaggerDirection,
    /// Optional: limit stagger to first N items
    pub limit: Option<usize>,
}

impl StaggerConfig {
    /// Create a new stagger config with delay between items
    pub fn new(delay_ms: u32, animation: impl Into<ElementAnimation>) -> Self {
        Self {
            delay_ms,
            animation: animation.into(),
            direction: StaggerDirection::Forward,
            limit: None,
        }
    }

    /// Stagger from last to first
    pub fn reverse(mut self) -> Self {
        self.direction = StaggerDirection::Reverse;
        self
    }

    /// Stagger from center outward
    pub fn from_center(mut self) -> Self {
        self.direction = StaggerDirection::FromCenter;
        self
    }

    /// Limit stagger to first N items
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Calculate delay for a specific child index
    pub fn delay_for_index(&self, index: usize, total: usize) -> u32 {
        let effective_index = match self.direction {
            StaggerDirection::Forward => index,
            StaggerDirection::Reverse => total.saturating_sub(1).saturating_sub(index),
            StaggerDirection::FromCenter => {
                let center = total / 2;
                if index <= center {
                    center - index
                } else {
                    index - center
                }
            }
        };

        // Apply limit if set
        let capped_index = if let Some(limit) = self.limit {
            effective_index.min(limit)
        } else {
            effective_index
        };

        self.delay_ms * capped_index as u32
    }
}

/// Shared animated value type for motion bindings (thread-safe)
pub type SharedAnimatedValue = std::sync::Arc<std::sync::Mutex<AnimatedValue>>;

/// Motion bindings for continuous animation driven by AnimatedValue
///
/// This struct holds references to animated values that are sampled every frame
/// during rendering, enabling smooth continuous animations.
#[derive(Clone, Default)]
pub struct MotionBindings {
    /// Animated X translation
    pub translate_x: Option<SharedAnimatedValue>,
    /// Animated Y translation
    pub translate_y: Option<SharedAnimatedValue>,
    /// Animated uniform scale
    pub scale: Option<SharedAnimatedValue>,
    /// Animated X scale
    pub scale_x: Option<SharedAnimatedValue>,
    /// Animated Y scale
    pub scale_y: Option<SharedAnimatedValue>,
    /// Animated rotation (degrees)
    pub rotation: Option<SharedAnimatedValue>,
    /// Animated opacity
    pub opacity: Option<SharedAnimatedValue>,
}

impl MotionBindings {
    /// Check if any bindings are set
    pub fn is_empty(&self) -> bool {
        self.translate_x.is_none()
            && self.translate_y.is_none()
            && self.scale.is_none()
            && self.scale_x.is_none()
            && self.scale_y.is_none()
            && self.rotation.is_none()
            && self.opacity.is_none()
    }

    /// Get the current translation from animated values
    ///
    /// Returns a translation transform for the tx/ty bindings.
    /// Scale and rotation should be queried separately for proper centered application.
    pub fn get_transform(&self) -> Option<Transform> {
        let tx = self.translate_x.as_ref().map(|v| v.lock().unwrap().get()).unwrap_or(0.0);
        let ty = self.translate_y.as_ref().map(|v| v.lock().unwrap().get()).unwrap_or(0.0);

        if tx.abs() > 0.001 || ty.abs() > 0.001 {
            Some(Transform::translate(tx, ty))
        } else {
            None
        }
    }

    /// Get the current scale values from animated bindings
    ///
    /// Returns (scale_x, scale_y) if any scale is bound.
    /// The renderer should apply this centered around the element.
    pub fn get_scale(&self) -> Option<(f32, f32)> {
        let scale = self.scale.as_ref().map(|v| v.lock().unwrap().get());
        let scale_x = self.scale_x.as_ref().map(|v| v.lock().unwrap().get());
        let scale_y = self.scale_y.as_ref().map(|v| v.lock().unwrap().get());

        if let Some(s) = scale {
            Some((s, s))
        } else if scale_x.is_some() || scale_y.is_some() {
            Some((scale_x.unwrap_or(1.0), scale_y.unwrap_or(1.0)))
        } else {
            None
        }
    }

    /// Get the current rotation from animated values (in degrees)
    ///
    /// The renderer should apply this centered around the element.
    pub fn get_rotation(&self) -> Option<f32> {
        self.rotation.as_ref().map(|v| v.lock().unwrap().get())
    }

    /// Get the current opacity from animated value
    pub fn get_opacity(&self) -> Option<f32> {
        self.opacity.as_ref().map(|v| v.lock().unwrap().get())
    }
}

/// Motion container for animations
///
/// Wraps child elements and applies animations. Supports:
/// - Entry/exit animations (one-time on mount/unmount)
/// - Continuous animations driven by `AnimatedValue` bindings
///
/// The container itself is transparent but can have layout properties
/// to control how children are arranged (flex direction, gap, etc.).
pub struct Motion {
    /// Children to animate (single or multiple)
    children: Vec<Box<dyn ElementBuilder>>,
    /// Entry animation
    enter: Option<ElementAnimation>,
    /// Exit animation
    exit: Option<ElementAnimation>,
    /// Stagger configuration for multiple children
    stagger_config: Option<StaggerConfig>,
    /// Layout style for the container
    style: Style,

    // =========================================================================
    // Continuous animation bindings (AnimatedValue driven)
    // =========================================================================

    /// Animated X translation
    translate_x: Option<SharedAnimatedValue>,
    /// Animated Y translation
    translate_y: Option<SharedAnimatedValue>,
    /// Animated uniform scale
    scale: Option<SharedAnimatedValue>,
    /// Animated X scale
    scale_x: Option<SharedAnimatedValue>,
    /// Animated Y scale
    scale_y: Option<SharedAnimatedValue>,
    /// Animated rotation (degrees)
    rotation: Option<SharedAnimatedValue>,
    /// Animated opacity
    opacity: Option<SharedAnimatedValue>,
}

/// Create a motion container
pub fn motion() -> Motion {
    Motion {
        children: Vec::new(),
        enter: None,
        exit: None,
        stagger_config: None,
        style: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Style::default()
        },
        translate_x: None,
        translate_y: None,
        scale: None,
        scale_x: None,
        scale_y: None,
        rotation: None,
        opacity: None,
    }
}

impl Motion {
    /// Set the child element to animate
    pub fn child(mut self, child: impl ElementBuilder + 'static) -> Self {
        // Store single child in children vec so it's returned by children_builders()
        self.children = vec![Box::new(child)];
        self
    }

    /// Add multiple children with stagger animation support
    pub fn children<I, E>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: ElementBuilder + 'static,
    {
        self.children = children
            .into_iter()
            .map(|c| Box::new(c) as Box<dyn ElementBuilder>)
            .collect();
        self
    }

    /// Set animation to play when element enters the tree
    pub fn enter_animation(mut self, animation: impl Into<ElementAnimation>) -> Self {
        self.enter = Some(animation.into());
        self
    }

    /// Set animation to play when element exits the tree
    pub fn exit_animation(mut self, animation: impl Into<ElementAnimation>) -> Self {
        self.exit = Some(animation.into());
        self
    }

    /// Enable stagger animations for multiple children
    pub fn stagger(mut self, config: StaggerConfig) -> Self {
        self.stagger_config = Some(config);
        self
    }

    // ========================================================================
    // Convenience methods for common animations
    // ========================================================================

    /// Fade in on enter
    pub fn fade_in(self, duration_ms: u32) -> Self {
        self.enter_animation(AnimationPreset::fade_in(duration_ms))
    }

    /// Fade out on exit
    pub fn fade_out(self, duration_ms: u32) -> Self {
        self.exit_animation(AnimationPreset::fade_out(duration_ms))
    }

    /// Scale in on enter
    pub fn scale_in(self, duration_ms: u32) -> Self {
        self.enter_animation(AnimationPreset::scale_in(duration_ms))
    }

    /// Scale out on exit
    pub fn scale_out(self, duration_ms: u32) -> Self {
        self.exit_animation(AnimationPreset::scale_out(duration_ms))
    }

    /// Bounce in on enter
    pub fn bounce_in(self, duration_ms: u32) -> Self {
        self.enter_animation(AnimationPreset::bounce_in(duration_ms))
    }

    /// Bounce out on exit
    pub fn bounce_out(self, duration_ms: u32) -> Self {
        self.exit_animation(AnimationPreset::bounce_out(duration_ms))
    }

    /// Slide in from direction
    pub fn slide_in(self, direction: SlideDirection, duration_ms: u32) -> Self {
        let distance = 50.0;
        let anim = match direction {
            SlideDirection::Left => AnimationPreset::slide_in_left(duration_ms, distance),
            SlideDirection::Right => AnimationPreset::slide_in_right(duration_ms, distance),
            SlideDirection::Top => AnimationPreset::slide_in_top(duration_ms, distance),
            SlideDirection::Bottom => AnimationPreset::slide_in_bottom(duration_ms, distance),
        };
        self.enter_animation(anim)
    }

    /// Slide out to direction
    pub fn slide_out(self, direction: SlideDirection, duration_ms: u32) -> Self {
        let distance = 50.0;
        let anim = match direction {
            SlideDirection::Left => AnimationPreset::slide_out_left(duration_ms, distance),
            SlideDirection::Right => AnimationPreset::slide_out_right(duration_ms, distance),
            SlideDirection::Top => AnimationPreset::slide_out_top(duration_ms, distance),
            SlideDirection::Bottom => AnimationPreset::slide_out_bottom(duration_ms, distance),
        };
        self.exit_animation(anim)
    }

    /// Pop in (scale with overshoot)
    pub fn pop_in(self, duration_ms: u32) -> Self {
        self.enter_animation(AnimationPreset::pop_in(duration_ms))
    }

    // ========================================================================
    // Continuous Animation Bindings (AnimatedValue driven)
    // ========================================================================

    /// Bind X translation to an AnimatedValue
    ///
    /// The motion element's X position will track this animated value.
    pub fn translate_x(mut self, value: SharedAnimatedValue) -> Self {
        self.translate_x = Some(value);
        self
    }

    /// Bind Y translation to an AnimatedValue
    ///
    /// The motion element's Y position will track this animated value.
    /// Perfect for pull-to-refresh, swipe gestures, etc.
    pub fn translate_y(mut self, value: SharedAnimatedValue) -> Self {
        self.translate_y = Some(value);
        self
    }

    /// Bind uniform scale to an AnimatedValue
    ///
    /// Scales both X and Y uniformly.
    pub fn scale(mut self, value: SharedAnimatedValue) -> Self {
        self.scale = Some(value);
        self
    }

    /// Bind X scale to an AnimatedValue
    pub fn scale_x(mut self, value: SharedAnimatedValue) -> Self {
        self.scale_x = Some(value);
        self
    }

    /// Bind Y scale to an AnimatedValue
    pub fn scale_y(mut self, value: SharedAnimatedValue) -> Self {
        self.scale_y = Some(value);
        self
    }

    /// Bind rotation to an AnimatedValue (in degrees)
    pub fn rotate(mut self, value: SharedAnimatedValue) -> Self {
        self.rotation = Some(value);
        self
    }

    /// Bind opacity to an AnimatedValue (0.0 to 1.0)
    pub fn opacity(mut self, value: SharedAnimatedValue) -> Self {
        self.opacity = Some(value);
        self
    }

    /// Check if any continuous animations are bound
    pub fn has_animated_bindings(&self) -> bool {
        self.translate_x.is_some()
            || self.translate_y.is_some()
            || self.scale.is_some()
            || self.scale_x.is_some()
            || self.scale_y.is_some()
            || self.rotation.is_some()
            || self.opacity.is_some()
    }

    /// Get the motion bindings for this element
    ///
    /// These bindings are stored in the RenderTree and sampled every frame
    /// during rendering to apply continuous animations.
    pub fn get_motion_bindings(&self) -> Option<MotionBindings> {
        if !self.has_animated_bindings() {
            return None;
        }

        Some(MotionBindings {
            translate_x: self.translate_x.clone(),
            translate_y: self.translate_y.clone(),
            scale: self.scale.clone(),
            scale_x: self.scale_x.clone(),
            scale_y: self.scale_y.clone(),
            rotation: self.rotation.clone(),
            opacity: self.opacity.clone(),
        })
    }

    // ========================================================================
    // Layout methods - control how children are arranged
    // ========================================================================

    /// Set the gap between children (in pixels)
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = taffy::Size {
            width: taffy::LengthPercentage::Length(gap),
            height: taffy::LengthPercentage::Length(gap),
        };
        self
    }

    /// Set flex direction to row
    pub fn flex_row(mut self) -> Self {
        self.style.flex_direction = FlexDirection::Row;
        self
    }

    /// Set flex direction to column
    pub fn flex_col(mut self) -> Self {
        self.style.flex_direction = FlexDirection::Column;
        self
    }

    /// Align items to center (cross-axis)
    pub fn items_center(mut self) -> Self {
        self.style.align_items = Some(taffy::AlignItems::Center);
        self
    }

    /// Align items to start (cross-axis)
    pub fn items_start(mut self) -> Self {
        self.style.align_items = Some(taffy::AlignItems::FlexStart);
        self
    }

    /// Justify content to center (main-axis)
    pub fn justify_center(mut self) -> Self {
        self.style.justify_content = Some(taffy::JustifyContent::Center);
        self
    }

    /// Justify content with space between (main-axis)
    pub fn justify_between(mut self) -> Self {
        self.style.justify_content = Some(taffy::JustifyContent::SpaceBetween);
        self
    }

    /// Get the enter animation if set
    pub fn get_enter_animation(&self) -> Option<&ElementAnimation> {
        self.enter.as_ref()
    }

    /// Get the exit animation if set
    pub fn get_exit_animation(&self) -> Option<&ElementAnimation> {
        self.exit.as_ref()
    }

    /// Get the stagger config if set
    pub fn get_stagger_config(&self) -> Option<&StaggerConfig> {
        self.stagger_config.as_ref()
    }

    /// Get the motion animation configuration for a child at given index
    ///
    /// Takes stagger into account to compute the correct delay for each child.
    pub fn motion_animation_for_child(&self, child_index: usize) -> Option<MotionAnimation> {
        let total_children = self.children.len();

        if total_children == 0 {
            return None;
        }

        // Calculate delay based on stagger config
        let delay_ms = if let Some(ref stagger) = self.stagger_config {
            stagger.delay_for_index(child_index, total_children)
        } else {
            0
        };

        // Get base animation (from stagger config or direct enter/exit)
        let enter_anim = if let Some(ref stagger) = self.stagger_config {
            Some(&stagger.animation.animation)
        } else {
            self.enter.as_ref().map(|e| &e.animation)
        };

        let exit_anim = self.exit.as_ref().map(|e| &e.animation);

        // Build MotionAnimation
        if let Some(enter) = enter_anim {
            let enter_from = enter
                .first_keyframe()
                .map(|kf| MotionKeyframe::from_keyframe_properties(&kf.properties));

            let mut motion = MotionAnimation {
                enter_from,
                enter_duration_ms: enter.duration_ms(),
                enter_delay_ms: delay_ms,
                exit_to: None,
                exit_duration_ms: 0,
            };

            if let Some(exit) = exit_anim {
                motion.exit_to = exit
                    .last_keyframe()
                    .map(|kf| MotionKeyframe::from_keyframe_properties(&kf.properties));
                motion.exit_duration_ms = exit.duration_ms();
            }

            Some(motion)
        } else if let Some(exit) = exit_anim {
            let exit_to = exit
                .last_keyframe()
                .map(|kf| MotionKeyframe::from_keyframe_properties(&kf.properties));

            Some(MotionAnimation {
                enter_from: None,
                enter_duration_ms: 0,
                enter_delay_ms: delay_ms,
                exit_to,
                exit_duration_ms: exit.duration_ms(),
            })
        } else {
            None
        }
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

impl ElementBuilder for Motion {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        // Create a container node with the configured style
        let node = tree.create_node(self.style.clone());

        // Build and add all children (stagger delay is computed later via motion_animation_for_child)
        for child in &self.children {
            let child_node = child.build(tree);
            tree.add_child(node, child_node);
        }

        node
    }

    fn render_props(&self) -> RenderProps {
        // Motion with animated bindings uses motion_bindings() instead of static props.
        // Return default props - the actual transform/opacity will be sampled at render time.
        RenderProps::default()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        // Return children vec - single child is now stored in children vec as well
        &self.children
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementTypeId::Motion
    }

    fn motion_animation_for_child(&self, child_index: usize) -> Option<MotionAnimation> {
        self.motion_animation_for_child(child_index)
    }

    fn motion_bindings(&self) -> Option<MotionBindings> {
        self.get_motion_bindings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stagger_delay_forward() {
        let config = StaggerConfig::new(50, AnimationPreset::fade_in(300));

        assert_eq!(config.delay_for_index(0, 5), 0);
        assert_eq!(config.delay_for_index(1, 5), 50);
        assert_eq!(config.delay_for_index(2, 5), 100);
        assert_eq!(config.delay_for_index(4, 5), 200);
    }

    #[test]
    fn test_stagger_delay_reverse() {
        let config = StaggerConfig::new(50, AnimationPreset::fade_in(300)).reverse();

        assert_eq!(config.delay_for_index(0, 5), 200);
        assert_eq!(config.delay_for_index(1, 5), 150);
        assert_eq!(config.delay_for_index(4, 5), 0);
    }

    #[test]
    fn test_stagger_delay_from_center() {
        let config = StaggerConfig::new(50, AnimationPreset::fade_in(300)).from_center();

        // For 5 items, center is index 2
        // Distances from center: [2, 1, 0, 1, 2]
        assert_eq!(config.delay_for_index(0, 5), 100); // 2 steps from center
        assert_eq!(config.delay_for_index(1, 5), 50); // 1 step from center
        assert_eq!(config.delay_for_index(2, 5), 0); // at center
        assert_eq!(config.delay_for_index(3, 5), 50); // 1 step from center
        assert_eq!(config.delay_for_index(4, 5), 100); // 2 steps from center
    }

    #[test]
    fn test_stagger_delay_with_limit() {
        let config = StaggerConfig::new(50, AnimationPreset::fade_in(300)).limit(3);

        assert_eq!(config.delay_for_index(0, 10), 0);
        assert_eq!(config.delay_for_index(3, 10), 150); // capped at limit
        assert_eq!(config.delay_for_index(5, 10), 150); // still capped
        assert_eq!(config.delay_for_index(9, 10), 150); // still capped
    }
}
