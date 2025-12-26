//! RenderState - Dynamic render properties separate from tree structure
//!
//! This module provides a clean separation between:
//! - **RenderTree**: Stable tree structure (rebuilt only when elements are added/removed)
//! - **RenderState**: Dynamic render properties (updated every frame without tree rebuild)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  UI Thread                                                       │
//! │  Event → State Change → Tree Rebuild (only structural changes)  │
//! └─────────────────────────────────────────────────────────────────┘
//!                               │
//!                               ▼
//!                     RenderTree (stable)
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  Render Loop (60fps)                                             │
//! │  1. Tick animations                                              │
//! │  2. Update RenderState from animations                           │
//! │  3. Render tree + state to GPU                                   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # What Goes Where
//!
//! | Property | RenderTree | RenderState |
//! |----------|------------|-------------|
//! | Element hierarchy | ✓ | |
//! | Layout constraints | ✓ | |
//! | Text content | ✓ | |
//! | Background color | | ✓ (animated) |
//! | Opacity | | ✓ (animated) |
//! | Transform | | ✓ (animated) |
//! | Cursor visibility | | ✓ (animated) |
//! | Scroll offset | | ✓ (animated) |
//! | Hover state | | ✓ (FSM) |
//! | Focus state | | ✓ (FSM) |

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use blinc_animation::{AnimationScheduler, SchedulerHandle, Spring, SpringConfig, SpringId};
use blinc_core::{Color, Transform};

use crate::element::{MotionAnimation, MotionKeyframe};
use crate::tree::LayoutNodeId;

/// State of a motion animation
#[derive(Clone, Debug)]
pub enum MotionState {
    /// Animation hasn't started yet (waiting for delay)
    Waiting { remaining_delay_ms: f32 },
    /// Animation is playing (enter animation)
    Entering { progress: f32, duration_ms: f32 },
    /// Element is fully visible (enter complete)
    Visible,
    /// Animation is playing (exit animation)
    Exiting { progress: f32, duration_ms: f32 },
    /// Element should be removed (exit complete)
    Removed,
}

impl Default for MotionState {
    fn default() -> Self {
        MotionState::Visible
    }
}

/// Active motion animation for a node
#[derive(Clone, Debug)]
pub struct ActiveMotion {
    /// The animation configuration
    pub config: MotionAnimation,
    /// Current state of the animation
    pub state: MotionState,
    /// Current interpolated values
    pub current: MotionKeyframe,
}

/// Dynamic render state for a single node
///
/// Contains all properties that can change without requiring a tree rebuild.
/// These properties are updated by animations or state machines.
#[derive(Clone, Debug)]
pub struct NodeRenderState {
    // =========================================================================
    // Animated visual properties
    // =========================================================================
    /// Current opacity (0.0 - 1.0)
    pub opacity: f32,

    /// Current background color (animated)
    pub background_color: Option<Color>,

    /// Current border color (animated)
    pub border_color: Option<Color>,

    /// Current transform (animated)
    pub transform: Option<Transform>,

    /// Current scale (animated, applied to transform)
    pub scale: f32,

    // =========================================================================
    // Animation handles (for tracking which properties are animating)
    // =========================================================================
    /// Spring ID for opacity animation
    pub opacity_spring: Option<SpringId>,

    /// Spring IDs for color animation (r, g, b, a)
    pub bg_color_springs: Option<[SpringId; 4]>,

    /// Spring IDs for transform (translate_x, translate_y, scale, rotate)
    pub transform_springs: Option<[SpringId; 4]>,

    // =========================================================================
    // Interaction state
    // =========================================================================
    /// Whether this node is currently hovered
    pub hovered: bool,

    /// Whether this node is currently focused
    pub focused: bool,

    /// Whether this node is currently pressed
    pub pressed: bool,

    // =========================================================================
    // Motion animation state
    // =========================================================================
    /// Active motion animation (enter/exit) for this node
    pub motion: Option<ActiveMotion>,
}

impl Default for NodeRenderState {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            background_color: None,
            border_color: None,
            transform: None,
            scale: 1.0,
            opacity_spring: None,
            bg_color_springs: None,
            transform_springs: None,
            hovered: false,
            focused: false,
            pressed: false,
            motion: None,
        }
    }
}

impl NodeRenderState {
    /// Create a new node render state with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any properties are currently animating
    pub fn is_animating(&self) -> bool {
        self.opacity_spring.is_some()
            || self.bg_color_springs.is_some()
            || self.transform_springs.is_some()
            || self.has_active_motion()
    }

    /// Check if this node has an active motion animation
    pub fn has_active_motion(&self) -> bool {
        if let Some(ref motion) = self.motion {
            !matches!(motion.state, MotionState::Visible | MotionState::Removed)
        } else {
            false
        }
    }
}

/// Overlay type for rendering on top of the tree
#[derive(Clone, Debug)]
pub enum Overlay {
    /// Text cursor overlay
    Cursor {
        /// Position (x, y)
        position: (f32, f32),
        /// Size (width, height)
        size: (f32, f32),
        /// Color
        color: Color,
        /// Current opacity (for blinking)
        opacity: f32,
    },
    /// Text selection overlay
    Selection {
        /// Selection rectangles (multiple for multi-line)
        rects: Vec<(f32, f32, f32, f32)>,
        /// Selection color
        color: Color,
    },
    /// Focus ring overlay
    FocusRing {
        /// Position (x, y)
        position: (f32, f32),
        /// Size (width, height)
        size: (f32, f32),
        /// Corner radius
        radius: f32,
        /// Ring color
        color: Color,
        /// Ring thickness
        thickness: f32,
    },
}

/// Global render state - updated every frame independently of tree rebuilds
///
/// This holds all dynamic render properties that change frequently:
/// - Animated colors, transforms, opacity
/// - Cursor blink state
/// - Scroll positions (from physics)
/// - Hover/focus visual state
pub struct RenderState {
    /// Per-node animated properties
    node_states: HashMap<LayoutNodeId, NodeRenderState>,

    /// Global overlays (cursors, selections, focus rings)
    overlays: Vec<Overlay>,

    /// Animation scheduler (shared with app)
    animations: Arc<Mutex<AnimationScheduler>>,

    /// Cursor blink state (global for all text inputs)
    cursor_visible: bool,

    /// Last cursor blink toggle time
    cursor_blink_time: u64,

    /// Cursor blink interval in ms
    cursor_blink_interval: u64,

    /// Last tick time (for calculating delta time)
    last_tick_time: Option<u64>,
}

impl RenderState {
    /// Create a new render state with the given animation scheduler
    pub fn new(animations: Arc<Mutex<AnimationScheduler>>) -> Self {
        Self {
            node_states: HashMap::new(),
            overlays: Vec::new(),
            animations,
            cursor_visible: true,
            cursor_blink_time: 0,
            cursor_blink_interval: 400,
            last_tick_time: None,
        }
    }

    /// Get animation scheduler handle for creating animations
    pub fn animation_handle(&self) -> SchedulerHandle {
        self.animations.lock().unwrap().handle()
    }

    /// Tick all animations and update render state
    ///
    /// Returns true if any animations are active (need another frame)
    pub fn tick(&mut self, current_time_ms: u64) -> bool {
        // Calculate delta time
        let dt_ms = if let Some(last_time) = self.last_tick_time {
            (current_time_ms.saturating_sub(last_time)) as f32
        } else {
            16.0 // Assume ~60fps for first frame
        };
        self.last_tick_time = Some(current_time_ms);

        // Tick the animation scheduler
        let animations_active = self.animations.lock().unwrap().tick();

        // Update cursor blink
        if current_time_ms >= self.cursor_blink_time + self.cursor_blink_interval {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_time = current_time_ms;
        }

        // Track if any motion animations are active
        let mut motion_active = false;

        // Update node states from their animation springs and motion animations
        let scheduler = self.animations.lock().unwrap();
        for (_node_id, state) in &mut self.node_states {
            // Update opacity from spring
            if let Some(spring_id) = state.opacity_spring {
                if let Some(value) = scheduler.get_spring_value(spring_id) {
                    state.opacity = value.clamp(0.0, 1.0);
                }
            }

            // Update background color from springs
            if let Some(springs) = state.bg_color_springs {
                let r = scheduler.get_spring_value(springs[0]).unwrap_or(0.0);
                let g = scheduler.get_spring_value(springs[1]).unwrap_or(0.0);
                let b = scheduler.get_spring_value(springs[2]).unwrap_or(0.0);
                let a = scheduler.get_spring_value(springs[3]).unwrap_or(1.0);
                state.background_color = Some(Color::rgba(r, g, b, a));
            }

            // Update transform from springs
            // Note: For now, we only support translation. Scale/rotation would need
            // matrix composition which Transform doesn't expose directly.
            if let Some(springs) = state.transform_springs {
                let tx = scheduler.get_spring_value(springs[0]).unwrap_or(0.0);
                let ty = scheduler.get_spring_value(springs[1]).unwrap_or(0.0);
                let scale = scheduler.get_spring_value(springs[2]).unwrap_or(1.0);
                let _rotate = scheduler.get_spring_value(springs[3]).unwrap_or(0.0);
                // TODO: Support scale/rotation when Transform supports composition
                state.transform = Some(Transform::translate(tx, ty));
                state.scale = scale;
            }

            // Update motion animation
            if let Some(ref mut motion) = state.motion {
                if Self::tick_motion(motion, dt_ms) {
                    motion_active = true;
                }
            }
        }

        // Update cursor overlays with blink state
        for overlay in &mut self.overlays {
            if let Overlay::Cursor { opacity, .. } = overlay {
                *opacity = if self.cursor_visible { 1.0 } else { 0.0 };
            }
        }

        animations_active || motion_active || self.has_overlays()
    }

    /// Tick a motion animation, returns true if still active
    fn tick_motion(motion: &mut ActiveMotion, dt_ms: f32) -> bool {
        match &mut motion.state {
            MotionState::Waiting { remaining_delay_ms } => {
                *remaining_delay_ms -= dt_ms;
                if *remaining_delay_ms <= 0.0 {
                    // Start enter animation
                    if motion.config.enter_from.is_some() && motion.config.enter_duration_ms > 0 {
                        tracing::debug!(
                            "Motion: Starting enter animation, duration={}ms",
                            motion.config.enter_duration_ms
                        );
                        motion.state = MotionState::Entering {
                            progress: 0.0,
                            duration_ms: motion.config.enter_duration_ms as f32,
                        };
                        // Initialize current to the "from" state
                        motion.current = motion.config.enter_from.clone().unwrap_or_default();
                    } else {
                        motion.state = MotionState::Visible;
                        motion.current = MotionKeyframe::default(); // Fully visible
                    }
                }
                true // Still animating
            }
            MotionState::Entering {
                progress,
                duration_ms,
            } => {
                *progress += dt_ms / *duration_ms;
                if *progress >= 1.0 {
                    motion.state = MotionState::Visible;
                    motion.current = MotionKeyframe::default(); // Fully visible (opacity=1, scale=1, etc.)
                    false // Done animating
                } else {
                    // Interpolate from enter_from to visible (default)
                    let from = motion
                        .config
                        .enter_from
                        .as_ref()
                        .cloned()
                        .unwrap_or_default();
                    let to = MotionKeyframe::default();
                    // Apply ease-out for enter animation
                    let eased = ease_out_cubic(*progress);
                    motion.current = from.lerp(&to, eased);
                    true // Still animating
                }
            }
            MotionState::Visible => false, // Not animating
            MotionState::Exiting {
                progress,
                duration_ms,
            } => {
                *progress += dt_ms / *duration_ms;
                if *progress >= 1.0 {
                    motion.state = MotionState::Removed;
                    motion.current = motion.config.exit_to.clone().unwrap_or_default();
                    false // Done animating
                } else {
                    // Interpolate from visible to exit_to
                    let from = MotionKeyframe::default();
                    let to = motion.config.exit_to.as_ref().cloned().unwrap_or_default();
                    // Apply ease-in for exit animation
                    let eased = ease_in_cubic(*progress);
                    motion.current = from.lerp(&to, eased);
                    true // Still animating
                }
            }
            MotionState::Removed => false, // Not animating
        }
    }

    /// Reset cursor blink (call when focus changes or user types)
    pub fn reset_cursor_blink(&mut self, current_time_ms: u64) {
        self.cursor_visible = true;
        self.cursor_blink_time = current_time_ms;
    }

    /// Set cursor blink interval
    pub fn set_cursor_blink_interval(&mut self, interval_ms: u64) {
        self.cursor_blink_interval = interval_ms;
    }

    /// Check if cursor is currently visible
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    // =========================================================================
    // Node State Management
    // =========================================================================

    /// Get or create render state for a node
    pub fn get_or_create(&mut self, node_id: LayoutNodeId) -> &mut NodeRenderState {
        self.node_states
            .entry(node_id)
            .or_insert_with(NodeRenderState::new)
    }

    /// Get render state for a node (if exists)
    pub fn get(&self, node_id: LayoutNodeId) -> Option<&NodeRenderState> {
        self.node_states.get(&node_id)
    }

    /// Get mutable render state for a node (if exists)
    pub fn get_mut(&mut self, node_id: LayoutNodeId) -> Option<&mut NodeRenderState> {
        self.node_states.get_mut(&node_id)
    }

    /// Remove render state for a node
    pub fn remove(&mut self, node_id: LayoutNodeId) {
        self.node_states.remove(&node_id);
    }

    /// Clear all node states (call when tree is completely rebuilt)
    pub fn clear_nodes(&mut self) {
        self.node_states.clear();
    }

    // =========================================================================
    // Animation Control
    // =========================================================================

    /// Animate opacity for a node
    pub fn animate_opacity(&mut self, node_id: LayoutNodeId, target: f32, config: SpringConfig) {
        // Get current values first
        let (current, old_spring) = {
            let state = self
                .node_states
                .entry(node_id)
                .or_insert_with(NodeRenderState::new);
            (state.opacity, state.opacity_spring.take())
        };

        // Remove existing spring if any
        if let Some(old_id) = old_spring {
            self.animations.lock().unwrap().remove_spring(old_id);
        }

        // Create new spring
        let mut spring = Spring::new(config, current);
        spring.set_target(target);
        let spring_id = self.animations.lock().unwrap().add_spring(spring);

        // Store the new spring id
        if let Some(state) = self.node_states.get_mut(&node_id) {
            state.opacity_spring = Some(spring_id);
        }
    }

    /// Animate background color for a node
    pub fn animate_background(
        &mut self,
        node_id: LayoutNodeId,
        target: Color,
        config: SpringConfig,
    ) {
        // Get current values first
        let (current, old_springs) = {
            let state = self
                .node_states
                .entry(node_id)
                .or_insert_with(NodeRenderState::new);
            let current = state.background_color.unwrap_or(Color::TRANSPARENT);
            (current, state.bg_color_springs.take())
        };

        // Remove existing springs if any
        if let Some(old_ids) = old_springs {
            let mut scheduler = self.animations.lock().unwrap();
            for id in old_ids {
                scheduler.remove_spring(id);
            }
        }

        // Create springs for r, g, b, a
        let springs = {
            let mut scheduler = self.animations.lock().unwrap();
            [
                {
                    let mut s = Spring::new(config, current.r);
                    s.set_target(target.r);
                    scheduler.add_spring(s)
                },
                {
                    let mut s = Spring::new(config, current.g);
                    s.set_target(target.g);
                    scheduler.add_spring(s)
                },
                {
                    let mut s = Spring::new(config, current.b);
                    s.set_target(target.b);
                    scheduler.add_spring(s)
                },
                {
                    let mut s = Spring::new(config, current.a);
                    s.set_target(target.a);
                    scheduler.add_spring(s)
                },
            ]
        };

        // Store the new spring ids
        if let Some(state) = self.node_states.get_mut(&node_id) {
            state.bg_color_springs = Some(springs);
        }
    }

    /// Set background color immediately (no animation)
    pub fn set_background(&mut self, node_id: LayoutNodeId, color: Color) {
        // Get old springs first
        let old_springs = {
            let state = self
                .node_states
                .entry(node_id)
                .or_insert_with(NodeRenderState::new);
            state.bg_color_springs.take()
        };

        // Remove any active animation
        if let Some(old_ids) = old_springs {
            let mut scheduler = self.animations.lock().unwrap();
            for id in old_ids {
                scheduler.remove_spring(id);
            }
        }

        // Set the color
        if let Some(state) = self.node_states.get_mut(&node_id) {
            state.background_color = Some(color);
        }
    }

    /// Set opacity immediately (no animation)
    pub fn set_opacity(&mut self, node_id: LayoutNodeId, opacity: f32) {
        // Get old spring first
        let old_spring = {
            let state = self
                .node_states
                .entry(node_id)
                .or_insert_with(NodeRenderState::new);
            state.opacity_spring.take()
        };

        // Remove any active animation
        if let Some(old_id) = old_spring {
            self.animations.lock().unwrap().remove_spring(old_id);
        }

        // Set the opacity
        if let Some(state) = self.node_states.get_mut(&node_id) {
            state.opacity = opacity;
        }
    }

    // =========================================================================
    // Overlay Management
    // =========================================================================

    /// Add a cursor overlay
    pub fn add_cursor(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.overlays.push(Overlay::Cursor {
            position: (x, y),
            size: (width, height),
            color,
            opacity: if self.cursor_visible { 1.0 } else { 0.0 },
        });
    }

    /// Add a selection overlay
    pub fn add_selection(&mut self, rects: Vec<(f32, f32, f32, f32)>, color: Color) {
        self.overlays.push(Overlay::Selection { rects, color });
    }

    /// Add a focus ring overlay
    pub fn add_focus_ring(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        color: Color,
        thickness: f32,
    ) {
        self.overlays.push(Overlay::FocusRing {
            position: (x, y),
            size: (width, height),
            radius,
            color,
            thickness,
        });
    }

    /// Clear all overlays (call before each frame's overlay collection)
    pub fn clear_overlays(&mut self) {
        self.overlays.clear();
    }

    /// Get all overlays for rendering
    pub fn overlays(&self) -> &[Overlay] {
        &self.overlays
    }

    /// Check if there are any overlays
    pub fn has_overlays(&self) -> bool {
        !self.overlays.is_empty()
    }

    // =========================================================================
    // Interaction State
    // =========================================================================

    /// Set hover state for a node
    pub fn set_hovered(&mut self, node_id: LayoutNodeId, hovered: bool) {
        self.get_or_create(node_id).hovered = hovered;
    }

    /// Set focus state for a node
    pub fn set_focused(&mut self, node_id: LayoutNodeId, focused: bool) {
        self.get_or_create(node_id).focused = focused;
    }

    /// Set pressed state for a node
    pub fn set_pressed(&mut self, node_id: LayoutNodeId, pressed: bool) {
        self.get_or_create(node_id).pressed = pressed;
    }

    /// Check if a node is hovered
    pub fn is_hovered(&self, node_id: LayoutNodeId) -> bool {
        self.get(node_id).map(|s| s.hovered).unwrap_or(false)
    }

    /// Check if a node is focused
    pub fn is_focused(&self, node_id: LayoutNodeId) -> bool {
        self.get(node_id).map(|s| s.focused).unwrap_or(false)
    }

    /// Check if a node is pressed
    pub fn is_pressed(&self, node_id: LayoutNodeId) -> bool {
        self.get(node_id).map(|s| s.pressed).unwrap_or(false)
    }

    // =========================================================================
    // Motion Animation Control
    // =========================================================================

    /// Start an enter motion animation for a node
    ///
    /// This is called when a node with motion config first appears in the tree.
    pub fn start_enter_motion(&mut self, node_id: LayoutNodeId, config: MotionAnimation) {
        let state = self.get_or_create(node_id);

        // Determine initial state based on delay
        let initial_state = if config.enter_delay_ms > 0 {
            MotionState::Waiting {
                remaining_delay_ms: config.enter_delay_ms as f32,
            }
        } else if config.enter_from.is_some() && config.enter_duration_ms > 0 {
            MotionState::Entering {
                progress: 0.0,
                duration_ms: config.enter_duration_ms as f32,
            }
        } else {
            MotionState::Visible
        };

        // Initial values come from enter_from (the starting state)
        let current = if matches!(initial_state, MotionState::Visible) {
            MotionKeyframe::default() // Already fully visible
        } else {
            config.enter_from.clone().unwrap_or_default()
        };

        state.motion = Some(ActiveMotion {
            config,
            state: initial_state,
            current,
        });
    }

    /// Start an exit motion animation for a node
    ///
    /// This is called when a node with motion config is about to be removed.
    pub fn start_exit_motion(&mut self, node_id: LayoutNodeId) {
        if let Some(state) = self.node_states.get_mut(&node_id) {
            if let Some(ref mut motion) = state.motion {
                if motion.config.exit_to.is_some() && motion.config.exit_duration_ms > 0 {
                    motion.state = MotionState::Exiting {
                        progress: 0.0,
                        duration_ms: motion.config.exit_duration_ms as f32,
                    };
                    motion.current = MotionKeyframe::default(); // Start from visible
                } else {
                    motion.state = MotionState::Removed;
                }
            }
        }
    }

    /// Get the current motion values for a node
    ///
    /// Returns the interpolated keyframe values if the node has an active motion.
    pub fn get_motion_values(&self, node_id: LayoutNodeId) -> Option<&MotionKeyframe> {
        self.get(node_id)
            .and_then(|s| s.motion.as_ref())
            .map(|m| &m.current)
    }

    /// Check if a node's motion animation is complete and should be removed
    pub fn is_motion_removed(&self, node_id: LayoutNodeId) -> bool {
        self.get(node_id)
            .and_then(|s| s.motion.as_ref())
            .map(|m| matches!(m.state, MotionState::Removed))
            .unwrap_or(false)
    }

    /// Check if any nodes have active motion animations
    pub fn has_active_motions(&self) -> bool {
        self.node_states.values().any(|s| s.has_active_motion())
    }
}

// ============================================================================
// Easing helper functions
// ============================================================================

/// Cubic ease-out (fast start, slow end) - good for enter animations
fn ease_out_cubic(t: f32) -> f32 {
    let t = 1.0 - t;
    1.0 - t * t * t
}

/// Cubic ease-in (slow start, fast end) - good for exit animations
fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_state_creation() {
        let scheduler = Arc::new(Mutex::new(AnimationScheduler::new()));
        let state = RenderState::new(scheduler);

        assert!(state.cursor_visible());
        assert!(!state.has_overlays());
    }

    #[test]
    fn test_node_render_state() {
        let scheduler = Arc::new(Mutex::new(AnimationScheduler::new()));
        let mut state = RenderState::new(scheduler);

        let node_id = LayoutNodeId::default();

        // Should auto-create on access
        state.set_hovered(node_id, true);
        assert!(state.is_hovered(node_id));

        state.set_opacity(node_id, 0.5);
        assert_eq!(state.get(node_id).unwrap().opacity, 0.5);
    }

    #[test]
    fn test_overlays() {
        let scheduler = Arc::new(Mutex::new(AnimationScheduler::new()));
        let mut state = RenderState::new(scheduler);

        state.add_cursor(10.0, 20.0, 2.0, 16.0, Color::WHITE);
        assert!(state.has_overlays());
        assert_eq!(state.overlays().len(), 1);

        state.clear_overlays();
        assert!(!state.has_overlays());
    }

    #[test]
    fn test_cursor_blink() {
        let scheduler = Arc::new(Mutex::new(AnimationScheduler::new()));
        let mut state = RenderState::new(scheduler);
        state.set_cursor_blink_interval(100);

        assert!(state.cursor_visible());

        // Tick past the blink interval
        state.tick(150);
        assert!(!state.cursor_visible());

        // Tick again
        state.tick(300);
        assert!(state.cursor_visible());
    }
}
