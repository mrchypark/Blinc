//! Keyframe animations
//!
//! This module provides both single-value keyframe animations and multi-property
//! keyframe animations for animating multiple properties simultaneously.

use crate::easing::Easing;

/// A single keyframe in a single-value animation
#[derive(Clone, Debug)]
pub struct Keyframe {
    /// Time position (0.0 to 1.0)
    pub time: f32,
    /// Target value at this keyframe
    pub value: f32,
    /// Easing function to use when transitioning TO this keyframe
    pub easing: Easing,
}

/// A keyframe-based animation (single value)
#[derive(Clone, Debug)]
pub struct KeyframeAnimation {
    duration_ms: u32,
    keyframes: Vec<Keyframe>,
    current_time: f32,
    playing: bool,
}

impl KeyframeAnimation {
    pub fn new(duration_ms: u32, keyframes: Vec<Keyframe>) -> Self {
        Self {
            duration_ms,
            keyframes,
            current_time: 0.0,
            playing: false,
        }
    }

    pub fn start(&mut self) {
        self.current_time = 0.0;
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn progress(&self) -> f32 {
        self.current_time / self.duration_ms as f32
    }

    /// Get the current interpolated value
    pub fn value(&self) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }

        let progress = self.progress().clamp(0.0, 1.0);

        // Find surrounding keyframes
        let mut prev_kf = &self.keyframes[0];
        let mut next_kf = &self.keyframes[0];

        for kf in &self.keyframes {
            if kf.time <= progress {
                prev_kf = kf;
            }
            if kf.time >= progress {
                next_kf = kf;
                break;
            }
        }

        if (prev_kf.time - next_kf.time).abs() < f32::EPSILON {
            return prev_kf.value;
        }

        // Interpolate between keyframes
        let local_progress = (progress - prev_kf.time) / (next_kf.time - prev_kf.time);
        let eased = next_kf.easing.apply(local_progress);

        prev_kf.value + (next_kf.value - prev_kf.value) * eased
    }

    /// Advance the animation by delta time (in milliseconds)
    pub fn tick(&mut self, dt_ms: f32) {
        if !self.playing {
            return;
        }

        self.current_time += dt_ms;

        if self.current_time >= self.duration_ms as f32 {
            self.current_time = self.duration_ms as f32;
            self.playing = false;
        }
    }

    /// Get the keyframes
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }
}

// ============================================================================
// Multi-Property Keyframe Animation
// ============================================================================

/// Properties that can be animated in a multi-property keyframe
#[derive(Clone, Debug, Default)]
pub struct KeyframeProperties {
    /// Opacity (0.0 to 1.0)
    pub opacity: Option<f32>,
    /// Scale X factor
    pub scale_x: Option<f32>,
    /// Scale Y factor
    pub scale_y: Option<f32>,
    /// Translation X in pixels
    pub translate_x: Option<f32>,
    /// Translation Y in pixels
    pub translate_y: Option<f32>,
    /// Rotation in degrees (Z-axis)
    pub rotate: Option<f32>,
    /// Rotation X in degrees (3D tilt)
    pub rotate_x: Option<f32>,
    /// Rotation Y in degrees (3D turn)
    pub rotate_y: Option<f32>,
    /// Perspective distance in pixels
    pub perspective: Option<f32>,
    /// 3D extrusion depth in pixels
    pub depth: Option<f32>,
    /// Z-axis translation in pixels (positive = toward viewer)
    pub translate_z: Option<f32>,
    /// Blend radius for smooth boolean operations (in pixels)
    pub blend_3d: Option<f32>,
    /// Clip-path inset [top%, right%, bottom%, left%]
    pub clip_inset: Option<[f32; 4]>,
}

impl KeyframeProperties {
    /// Create properties with only opacity set
    pub fn opacity(value: f32) -> Self {
        Self {
            opacity: Some(value),
            ..Default::default()
        }
    }

    /// Create properties with uniform scale
    pub fn scale(value: f32) -> Self {
        Self {
            scale_x: Some(value),
            scale_y: Some(value),
            ..Default::default()
        }
    }

    /// Create properties with translation
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            translate_x: Some(x),
            translate_y: Some(y),
            ..Default::default()
        }
    }

    /// Create properties with rotation
    pub fn rotation(degrees: f32) -> Self {
        Self {
            rotate: Some(degrees),
            ..Default::default()
        }
    }

    /// Builder: set opacity
    pub fn with_opacity(mut self, value: f32) -> Self {
        self.opacity = Some(value);
        self
    }

    /// Builder: set uniform scale
    pub fn with_scale(mut self, value: f32) -> Self {
        self.scale_x = Some(value);
        self.scale_y = Some(value);
        self
    }

    /// Builder: set scale x and y separately
    pub fn with_scale_xy(mut self, x: f32, y: f32) -> Self {
        self.scale_x = Some(x);
        self.scale_y = Some(y);
        self
    }

    /// Builder: set translation
    pub fn with_translate(mut self, x: f32, y: f32) -> Self {
        self.translate_x = Some(x);
        self.translate_y = Some(y);
        self
    }

    /// Builder: set rotation
    pub fn with_rotate(mut self, degrees: f32) -> Self {
        self.rotate = Some(degrees);
        self
    }

    /// Builder: set X rotation (3D tilt)
    pub fn with_rotate_x(mut self, degrees: f32) -> Self {
        self.rotate_x = Some(degrees);
        self
    }

    /// Builder: set Y rotation (3D turn)
    pub fn with_rotate_y(mut self, degrees: f32) -> Self {
        self.rotate_y = Some(degrees);
        self
    }

    /// Builder: set perspective distance
    pub fn with_perspective(mut self, px: f32) -> Self {
        self.perspective = Some(px);
        self
    }

    /// Builder: set 3D depth
    pub fn with_depth(mut self, px: f32) -> Self {
        self.depth = Some(px);
        self
    }

    /// Builder: set translate-z
    pub fn with_translate_z(mut self, px: f32) -> Self {
        self.translate_z = Some(px);
        self
    }

    /// Builder: set 3D blend radius
    pub fn with_blend_3d(mut self, px: f32) -> Self {
        self.blend_3d = Some(px);
        self
    }

    /// Interpolate between two property sets
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            opacity: lerp_opt(self.opacity, other.opacity, t),
            scale_x: lerp_opt(self.scale_x, other.scale_x, t),
            scale_y: lerp_opt(self.scale_y, other.scale_y, t),
            translate_x: lerp_opt(self.translate_x, other.translate_x, t),
            translate_y: lerp_opt(self.translate_y, other.translate_y, t),
            rotate: lerp_opt(self.rotate, other.rotate, t),
            rotate_x: lerp_opt(self.rotate_x, other.rotate_x, t),
            rotate_y: lerp_opt(self.rotate_y, other.rotate_y, t),
            perspective: lerp_opt(self.perspective, other.perspective, t),
            depth: lerp_opt(self.depth, other.depth, t),
            translate_z: lerp_opt(self.translate_z, other.translate_z, t),
            blend_3d: lerp_opt(self.blend_3d, other.blend_3d, t),
            clip_inset: lerp_opt_array4(self.clip_inset, other.clip_inset, t),
        }
    }

    /// Get the resolved opacity (defaults to 1.0 if not set)
    pub fn resolved_opacity(&self) -> f32 {
        self.opacity.unwrap_or(1.0)
    }

    /// Get the resolved scale (defaults to 1.0 if not set)
    pub fn resolved_scale(&self) -> (f32, f32) {
        (self.scale_x.unwrap_or(1.0), self.scale_y.unwrap_or(1.0))
    }

    /// Get the resolved translation (defaults to 0.0 if not set)
    pub fn resolved_translate(&self) -> (f32, f32) {
        (
            self.translate_x.unwrap_or(0.0),
            self.translate_y.unwrap_or(0.0),
        )
    }

    /// Get the resolved rotation (defaults to 0.0 if not set)
    pub fn resolved_rotate(&self) -> f32 {
        self.rotate.unwrap_or(0.0)
    }

    /// Get the resolved X rotation in degrees (defaults to 0.0)
    pub fn resolved_rotate_x(&self) -> f32 {
        self.rotate_x.unwrap_or(0.0)
    }

    /// Get the resolved Y rotation in degrees (defaults to 0.0)
    pub fn resolved_rotate_y(&self) -> f32 {
        self.rotate_y.unwrap_or(0.0)
    }

    /// Get the resolved perspective distance (defaults to 0.0 = no perspective)
    pub fn resolved_perspective(&self) -> f32 {
        self.perspective.unwrap_or(0.0)
    }

    /// Get the resolved 3D depth (defaults to 0.0 = flat)
    pub fn resolved_depth(&self) -> f32 {
        self.depth.unwrap_or(0.0)
    }
}

/// Helper to interpolate optional values
fn lerp_opt(a: Option<f32>, b: Option<f32>, t: f32) -> Option<f32> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a + (b - a) * t),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Helper to interpolate optional [f32; 4] arrays
fn lerp_opt_array4(a: Option<[f32; 4]>, b: Option<[f32; 4]>, t: f32) -> Option<[f32; 4]> {
    match (a, b) {
        (Some(a), Some(b)) => Some([
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ]),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// A keyframe with multiple animated properties
#[derive(Clone, Debug)]
pub struct MultiKeyframe {
    /// Time position (0.0 to 1.0)
    pub time: f32,
    /// Properties at this keyframe
    pub properties: KeyframeProperties,
    /// Easing function to use when transitioning TO this keyframe
    pub easing: Easing,
}

impl MultiKeyframe {
    /// Create a new multi-property keyframe
    pub fn new(time: f32, properties: KeyframeProperties, easing: Easing) -> Self {
        Self {
            time,
            properties,
            easing,
        }
    }
}

/// Playback direction for animations
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayDirection {
    /// Play forward (0.0 -> 1.0)
    #[default]
    Forward,
    /// Play in reverse (1.0 -> 0.0)
    Reverse,
    /// Alternate between forward and reverse each iteration
    Alternate,
}

/// Fill mode determines the animation state before/after playback
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillMode {
    /// No fill - reset to initial state after animation
    #[default]
    None,
    /// Hold the final keyframe value after animation completes
    Forwards,
    /// Apply the first keyframe value before animation starts
    Backwards,
    /// Apply both forwards and backwards fill
    Both,
}

/// Multi-property keyframe animation
#[derive(Clone, Debug)]
pub struct MultiKeyframeAnimation {
    /// Duration in milliseconds
    duration_ms: u32,
    /// Keyframes sorted by time
    keyframes: Vec<MultiKeyframe>,
    /// Current time in milliseconds
    current_time: f32,
    /// Whether animation is playing
    playing: bool,
    /// Playback direction
    direction: PlayDirection,
    /// Fill mode
    fill_mode: FillMode,
    /// Number of iterations (-1 for infinite)
    iterations: i32,
    /// Current iteration count
    current_iteration: i32,
    /// Whether currently playing in reverse (for Alternate)
    reversed: bool,
    /// Delay before animation starts (ms)
    delay_ms: u32,
}

impl MultiKeyframeAnimation {
    /// Create a new multi-property animation with given duration
    pub fn new(duration_ms: u32) -> Self {
        Self {
            duration_ms,
            keyframes: Vec::new(),
            current_time: 0.0,
            playing: false,
            direction: PlayDirection::Forward,
            fill_mode: FillMode::Forwards,
            iterations: 1,
            current_iteration: 0,
            reversed: false,
            delay_ms: 0,
        }
    }

    /// Add a keyframe to the animation (builder pattern)
    pub fn keyframe(mut self, time: f32, properties: KeyframeProperties, easing: Easing) -> Self {
        self.keyframes
            .push(MultiKeyframe::new(time, properties, easing));
        // Keep keyframes sorted by time
        self.keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self
    }

    /// Set playback direction
    pub fn direction(mut self, direction: PlayDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set fill mode
    pub fn fill_mode(mut self, fill_mode: FillMode) -> Self {
        self.fill_mode = fill_mode;
        self
    }

    /// Set number of iterations (-1 for infinite)
    pub fn iterations(mut self, count: i32) -> Self {
        self.iterations = count;
        self
    }

    /// Set delay before animation starts
    pub fn delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    // =========================================================================
    // Mutating setters (for post-construction configuration)
    // =========================================================================

    /// Set playback direction (mutating)
    pub fn set_direction(&mut self, direction: PlayDirection) {
        self.direction = direction;
    }

    /// Set fill mode (mutating)
    pub fn set_fill_mode(&mut self, fill_mode: FillMode) {
        self.fill_mode = fill_mode;
    }

    /// Set number of iterations (mutating, -1 for infinite)
    pub fn set_iterations(&mut self, count: i32) {
        self.iterations = count;
    }

    /// Set delay before animation starts (mutating)
    pub fn set_delay(&mut self, delay_ms: u32) {
        self.delay_ms = delay_ms;
    }

    /// Set reversed state (mutating)
    ///
    /// When true, animation plays in reverse direction for the current iteration.
    /// Used internally for Alternate direction and for AlternateReverse start.
    pub fn set_reversed(&mut self, reversed: bool) {
        self.reversed = reversed;
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.current_time = -(self.delay_ms as f32);
        self.current_iteration = 0;
        self.reversed = self.direction == PlayDirection::Reverse;
        self.playing = true;
    }

    /// Stop the animation
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Check if the animation is currently playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Get the current progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.current_time < 0.0 {
            return 0.0;
        }
        (self.current_time / self.duration_ms as f32).clamp(0.0, 1.0)
    }

    /// Get the current interpolated properties
    pub fn current_properties(&self) -> KeyframeProperties {
        if self.keyframes.is_empty() {
            return KeyframeProperties::default();
        }

        // Handle delay period
        if self.current_time < 0.0 {
            return match self.fill_mode {
                FillMode::Backwards | FillMode::Both => self.keyframes[0].properties.clone(),
                _ => KeyframeProperties::default(),
            };
        }

        let mut progress = self.progress();

        // Apply reverse if needed
        if self.reversed {
            progress = 1.0 - progress;
        }

        // Find surrounding keyframes
        let mut prev_kf = &self.keyframes[0];
        let mut next_kf = &self.keyframes[0];

        for kf in &self.keyframes {
            if kf.time <= progress {
                prev_kf = kf;
            }
            if kf.time >= progress {
                next_kf = kf;
                break;
            }
        }

        if (prev_kf.time - next_kf.time).abs() < f32::EPSILON {
            return prev_kf.properties.clone();
        }

        // Interpolate between keyframes
        let local_progress = (progress - prev_kf.time) / (next_kf.time - prev_kf.time);
        let eased = next_kf.easing.apply(local_progress);

        prev_kf.properties.lerp(&next_kf.properties, eased)
    }

    /// Advance the animation by delta time (in milliseconds)
    pub fn tick(&mut self, dt_ms: f32) {
        if !self.playing {
            return;
        }

        self.current_time += dt_ms;

        // Check if we've completed an iteration
        if self.current_time >= self.duration_ms as f32 {
            self.current_iteration += 1;

            // Check if we should continue (infinite or more iterations remaining)
            if self.iterations < 0 || self.current_iteration < self.iterations {
                // Reset for next iteration
                self.current_time = 0.0;

                // Handle alternate direction
                if self.direction == PlayDirection::Alternate {
                    self.reversed = !self.reversed;
                }
            } else {
                // Animation complete
                self.current_time = self.duration_ms as f32;
                self.playing = false;
            }
        }
    }

    /// Get the duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    /// Get the delay in milliseconds
    pub fn delay_ms(&self) -> u32 {
        self.delay_ms
    }

    /// Get total duration including delay
    pub fn total_duration_ms(&self) -> u32 {
        self.delay_ms + self.duration_ms
    }

    /// Get the first keyframe (if any)
    pub fn first_keyframe(&self) -> Option<&MultiKeyframe> {
        self.keyframes.first()
    }

    /// Get the last keyframe (if any)
    pub fn last_keyframe(&self) -> Option<&MultiKeyframe> {
        self.keyframes.last()
    }

    /// Get all keyframes
    pub fn keyframes(&self) -> &[MultiKeyframe] {
        &self.keyframes
    }

    /// Sample the animation at a specific progress (0.0 to 1.0)
    ///
    /// This is useful for externally-timed animations where you manage
    /// the progress yourself rather than using tick().
    pub fn sample_at(&self, progress: f32) -> KeyframeProperties {
        if self.keyframes.is_empty() {
            return KeyframeProperties::default();
        }

        let progress = progress.clamp(0.0, 1.0);

        // Find surrounding keyframes
        let mut prev_kf = &self.keyframes[0];
        let mut next_kf = &self.keyframes[0];

        for kf in &self.keyframes {
            if kf.time <= progress {
                prev_kf = kf;
            }
            if kf.time >= progress {
                next_kf = kf;
                break;
            }
        }

        if (prev_kf.time - next_kf.time).abs() < f32::EPSILON {
            return prev_kf.properties.clone();
        }

        // Interpolate between keyframes
        let local_progress = (progress - prev_kf.time) / (next_kf.time - prev_kf.time);
        let eased = next_kf.easing.apply(local_progress);

        prev_kf.properties.lerp(&next_kf.properties, eased)
    }
}

impl Default for MultiKeyframeAnimation {
    fn default() -> Self {
        Self::new(300)
    }
}

// ============================================================================
// Keyframe Track Builder - Fluent API for single-value keyframe animations
// ============================================================================

/// A point in time with a value for keyframe animation
#[derive(Clone, Debug)]
pub struct KeyframePoint {
    /// Time in milliseconds
    pub time_ms: u32,
    /// Value at this time
    pub value: f32,
    /// Easing to use when transitioning TO this point
    pub easing: Easing,
}

/// Builder for creating keyframe animations with a fluent API
///
/// # Example
///
/// ```ignore
/// let track = KeyframeTrackBuilder::new()
///     .at(0, 0.8)
///     .at(800, 1.2)
///     .ease(Easing::EaseInOut)
///     .ping_pong()
///     .loop_infinite()
///     .build();
/// ```
#[derive(Clone, Debug)]
pub struct KeyframeTrackBuilder {
    /// Keyframe points (time_ms, value)
    points: Vec<KeyframePoint>,
    /// Default easing for all keyframes (can be overridden per-keyframe)
    default_easing: Easing,
    /// Play direction
    direction: PlayDirection,
    /// Number of iterations (-1 for infinite)
    iterations: i32,
    /// Delay before starting (ms)
    delay_ms: u32,
    /// Fill mode
    fill_mode: FillMode,
    /// Whether to auto-start
    auto_start: bool,
}

impl Default for KeyframeTrackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyframeTrackBuilder {
    /// Create a new keyframe track builder
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            default_easing: Easing::Linear,
            direction: PlayDirection::Forward,
            iterations: 1,
            delay_ms: 0,
            fill_mode: FillMode::Forwards,
            auto_start: false,
        }
    }

    /// Add a keyframe at a specific time (in milliseconds)
    pub fn at(mut self, time_ms: u32, value: f32) -> Self {
        self.points.push(KeyframePoint {
            time_ms,
            value,
            easing: self.default_easing,
        });
        self
    }

    /// Add a keyframe with specific easing
    pub fn at_with_ease(mut self, time_ms: u32, value: f32, easing: Easing) -> Self {
        self.points.push(KeyframePoint {
            time_ms,
            value,
            easing,
        });
        self
    }

    /// Set the default easing for all keyframes
    ///
    /// This applies to keyframes added after this call, and updates
    /// existing keyframes that haven't had explicit easing set.
    pub fn ease(mut self, easing: Easing) -> Self {
        self.default_easing = easing;
        // Update existing points that use the default
        for point in &mut self.points {
            point.easing = easing;
        }
        self
    }

    /// Enable ping-pong mode (alternate between forward and reverse)
    ///
    /// The animation plays forward, then backward, creating a smooth
    /// back-and-forth effect without needing duplicate keyframes.
    pub fn ping_pong(mut self) -> Self {
        self.direction = PlayDirection::Alternate;
        self
    }

    /// Alias for ping_pong()
    pub fn alternate(self) -> Self {
        self.ping_pong()
    }

    /// Play in reverse direction
    pub fn reverse(mut self) -> Self {
        self.direction = PlayDirection::Reverse;
        self
    }

    /// Set the number of loop iterations
    pub fn loop_count(mut self, count: i32) -> Self {
        self.iterations = count;
        self
    }

    /// Loop infinitely
    pub fn loop_infinite(mut self) -> Self {
        self.iterations = -1;
        self
    }

    /// Set delay before animation starts (in milliseconds)
    pub fn delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Set fill mode (what happens before/after animation)
    pub fn fill(mut self, fill_mode: FillMode) -> Self {
        self.fill_mode = fill_mode;
        self
    }

    /// Auto-start the animation when built
    pub fn start(mut self) -> Self {
        self.auto_start = true;
        self
    }

    /// Get the total duration based on keyframe times
    pub fn duration_ms(&self) -> u32 {
        self.points.iter().map(|p| p.time_ms).max().unwrap_or(0)
    }

    /// Build into a KeyframeTrack
    pub fn build(mut self) -> KeyframeTrack {
        // Sort points by time
        self.points.sort_by_key(|p| p.time_ms);

        let duration = self.duration_ms();

        // Convert to normalized keyframes (time as 0.0-1.0)
        let keyframes: Vec<Keyframe> = self
            .points
            .iter()
            .map(|p| {
                let time = if duration > 0 {
                    p.time_ms as f32 / duration as f32
                } else {
                    0.0
                };
                Keyframe {
                    time,
                    value: p.value,
                    easing: p.easing,
                }
            })
            .collect();

        let mut animation = KeyframeAnimation::new(duration, keyframes);

        if self.auto_start {
            animation.start();
        }

        KeyframeTrack {
            animation,
            direction: self.direction,
            iterations: self.iterations,
            current_iteration: 0,
            reversed: self.direction == PlayDirection::Reverse,
            delay_ms: self.delay_ms,
            fill_mode: self.fill_mode,
            duration_ms: duration,
        }
    }
}

/// A complete keyframe track with playback controls
///
/// Wraps KeyframeAnimation with additional features like ping-pong,
/// looping, and fill modes.
#[derive(Clone, Debug)]
pub struct KeyframeTrack {
    animation: KeyframeAnimation,
    direction: PlayDirection,
    iterations: i32,
    current_iteration: i32,
    reversed: bool,
    #[allow(dead_code)] // Reserved for future delay support
    delay_ms: u32,
    #[allow(dead_code)] // Reserved for future fill mode support
    fill_mode: FillMode,
    duration_ms: u32,
}

impl KeyframeTrack {
    /// Create a new builder
    pub fn builder() -> KeyframeTrackBuilder {
        KeyframeTrackBuilder::new()
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.current_iteration = 0;
        self.reversed = self.direction == PlayDirection::Reverse;
        self.animation.start();
    }

    /// Stop the animation
    pub fn stop(&mut self) {
        self.animation.stop();
    }

    /// Restart the animation from the beginning
    pub fn restart(&mut self) {
        self.start();
    }

    /// Check if the animation is playing
    pub fn is_playing(&self) -> bool {
        self.animation.is_playing() || self.should_continue()
    }

    /// Check if we should continue to next iteration
    fn should_continue(&self) -> bool {
        self.iterations < 0 || self.current_iteration < self.iterations
    }

    /// Get the current value with ping-pong and iteration support
    pub fn value(&self) -> f32 {
        let keyframes = self.animation.keyframes();
        if keyframes.is_empty() {
            return 0.0;
        }

        let progress = self.animation.progress().clamp(0.0, 1.0);

        // Apply reverse if in ping-pong mode and on reverse phase
        let effective_progress = if self.reversed {
            1.0 - progress
        } else {
            progress
        };

        // Find surrounding keyframes
        let mut prev_kf = &keyframes[0];
        let mut next_kf = &keyframes[0];

        for kf in keyframes {
            if kf.time <= effective_progress {
                prev_kf = kf;
            }
            if kf.time >= effective_progress {
                next_kf = kf;
                break;
            }
        }

        if (prev_kf.time - next_kf.time).abs() < f32::EPSILON {
            return prev_kf.value;
        }

        // Interpolate between keyframes
        let local_progress = (effective_progress - prev_kf.time) / (next_kf.time - prev_kf.time);
        let eased = next_kf.easing.apply(local_progress);

        prev_kf.value + (next_kf.value - prev_kf.value) * eased
    }

    /// Get the current progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        let progress = self.animation.progress();
        if self.reversed {
            1.0 - progress
        } else {
            progress
        }
    }

    /// Advance the animation by delta time (in milliseconds)
    pub fn tick(&mut self, dt_ms: f32) {
        self.animation.tick(dt_ms);

        // Check if we've completed an iteration
        if self.animation.progress() >= 1.0 && !self.animation.is_playing() {
            self.current_iteration += 1;

            // Check if we should continue
            if self.iterations < 0 || self.current_iteration < self.iterations {
                // Reset for next iteration
                self.animation.start();

                // Handle alternate direction
                if self.direction == PlayDirection::Alternate {
                    self.reversed = !self.reversed;
                }
            }
        }
    }

    /// Get the duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }
}
