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
    /// Rotation in degrees
    pub rotate: Option<f32>,
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

    /// Interpolate between two property sets
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            opacity: lerp_opt(self.opacity, other.opacity, t),
            scale_x: lerp_opt(self.scale_x, other.scale_x, t),
            scale_y: lerp_opt(self.scale_y, other.scale_y, t),
            translate_x: lerp_opt(self.translate_x, other.translate_x, t),
            translate_y: lerp_opt(self.translate_y, other.translate_y, t),
            rotate: lerp_opt(self.rotate, other.rotate, t),
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
}

impl Default for MultiKeyframeAnimation {
    fn default() -> Self {
        Self::new(300)
    }
}
