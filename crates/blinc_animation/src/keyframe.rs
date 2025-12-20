//! Keyframe animations

use crate::easing::Easing;

/// A single keyframe in an animation
#[derive(Clone, Debug)]
pub struct Keyframe {
    /// Time position (0.0 to 1.0)
    pub time: f32,
    /// Target value at this keyframe
    pub value: f32,
    /// Easing function to use when transitioning TO this keyframe
    pub easing: Easing,
}

/// A keyframe-based animation
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
