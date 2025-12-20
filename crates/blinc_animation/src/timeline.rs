//! Timeline orchestration for multiple animations

use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct TimelineEntryId;
}

/// An entry in a timeline
struct TimelineEntry {
    /// Offset in milliseconds from timeline start
    offset_ms: i32,
    /// Duration of the animation
    duration_ms: u32,
    /// Animation values (for now, just start/end)
    start_value: f32,
    end_value: f32,
}

/// A timeline that orchestrates multiple animations
pub struct Timeline {
    entries: SlotMap<TimelineEntryId, TimelineEntry>,
    current_time: f32,
    duration_ms: u32,
    playing: bool,
    loop_count: i32, // -1 for infinite
    current_loop: i32,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            entries: SlotMap::with_key(),
            current_time: 0.0,
            duration_ms: 0,
            playing: false,
            loop_count: 1,
            current_loop: 0,
        }
    }

    /// Add an animation to the timeline at a given offset
    pub fn add(
        &mut self,
        offset_ms: i32,
        duration_ms: u32,
        start_value: f32,
        end_value: f32,
    ) -> TimelineEntryId {
        let id = self.entries.insert(TimelineEntry {
            offset_ms,
            duration_ms,
            start_value,
            end_value,
        });

        // Update total duration
        let end_time = (offset_ms.max(0) as u32) + duration_ms;
        self.duration_ms = self.duration_ms.max(end_time);

        id
    }

    pub fn start(&mut self) {
        self.current_time = 0.0;
        self.current_loop = 0;
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn set_loop(&mut self, count: i32) {
        self.loop_count = count;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Advance the timeline
    pub fn tick(&mut self, dt_ms: f32) {
        if !self.playing {
            return;
        }

        self.current_time += dt_ms;

        if self.current_time >= self.duration_ms as f32 {
            if self.loop_count == -1 || self.current_loop < self.loop_count - 1 {
                self.current_time = 0.0;
                self.current_loop += 1;
            } else {
                self.current_time = self.duration_ms as f32;
                self.playing = false;
            }
        }
    }

    /// Get the current value for an animation entry
    pub fn value(&self, id: TimelineEntryId) -> Option<f32> {
        let entry = self.entries.get(id)?;

        let local_time = self.current_time - entry.offset_ms as f32;

        if local_time < 0.0 {
            return Some(entry.start_value);
        }

        if local_time >= entry.duration_ms as f32 {
            return Some(entry.end_value);
        }

        let progress = local_time / entry.duration_ms as f32;
        Some(entry.start_value + (entry.end_value - entry.start_value) * progress)
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}
