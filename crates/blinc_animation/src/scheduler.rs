//! Animation scheduler
//!
//! Manages all active animations and updates them each frame.

use crate::spring::Spring;
use slotmap::{new_key_type, SlotMap};
use std::time::Instant;

new_key_type! {
    pub struct SpringId;
    pub struct KeyframeId;
    pub struct TimelineId;
}

/// The animation scheduler that ticks all active animations
pub struct AnimationScheduler {
    springs: SlotMap<SpringId, Spring>,
    last_frame: Instant,
    target_fps: u32,
}

impl AnimationScheduler {
    pub fn new() -> Self {
        Self {
            springs: SlotMap::with_key(),
            last_frame: Instant::now(),
            target_fps: 120,
        }
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
    }

    pub fn add_spring(&mut self, spring: Spring) -> SpringId {
        self.springs.insert(spring)
    }

    pub fn get_spring(&self, id: SpringId) -> Option<&Spring> {
        self.springs.get(id)
    }

    pub fn get_spring_mut(&mut self, id: SpringId) -> Option<&mut Spring> {
        self.springs.get_mut(id)
    }

    pub fn remove_spring(&mut self, id: SpringId) -> Option<Spring> {
        self.springs.remove(id)
    }

    /// Tick all animations
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Update all springs
        for (_, spring) in self.springs.iter_mut() {
            spring.step(dt);
        }

        // TODO: Update keyframe animations
        // TODO: Update timelines
    }

    /// Check if any animations are still active
    pub fn has_active_animations(&self) -> bool {
        self.springs.iter().any(|(_, s)| !s.is_settled())
    }

    /// Iterate over all springs (immutable)
    pub fn springs_iter(&self) -> impl Iterator<Item = (SpringId, &Spring)> {
        self.springs.iter()
    }

    /// Iterate over all springs (mutable)
    pub fn springs_iter_mut(&mut self) -> impl Iterator<Item = (SpringId, &mut Spring)> {
        self.springs.iter_mut()
    }

    /// Get the number of springs in the scheduler
    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }
}

impl Default for AnimationScheduler {
    fn default() -> Self {
        Self::new()
    }
}
