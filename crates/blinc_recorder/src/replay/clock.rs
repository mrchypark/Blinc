//! Virtual clock for deterministic replay timing.
//!
//! Provides a controllable time source that can be paused, seeked,
//! and played at different speeds.

use crate::Timestamp;
use std::time::{Duration, Instant};

/// A virtual clock for controlling replay timing.
///
/// The virtual clock can be:
/// - Paused/resumed
/// - Seeked to any position
/// - Played at different speeds (0.5x, 1x, 2x, etc.)
#[derive(Debug)]
pub struct VirtualClock {
    /// Current virtual time position.
    position: Timestamp,
    /// Whether the clock is running.
    running: bool,
    /// Playback speed multiplier (1.0 = normal, 2.0 = 2x speed).
    speed: f64,
    /// Real-time instant when the clock was last updated.
    last_update: Option<Instant>,
    /// Duration of the recording (for bounds checking).
    duration: Timestamp,
}

impl VirtualClock {
    /// Create a new virtual clock.
    pub fn new(duration: Timestamp) -> Self {
        Self {
            position: Timestamp::zero(),
            running: false,
            speed: 1.0,
            last_update: None,
            duration,
        }
    }

    /// Get the current virtual time position.
    pub fn position(&self) -> Timestamp {
        self.position
    }

    /// Get the total duration.
    pub fn duration(&self) -> Timestamp {
        self.duration
    }

    /// Check if the clock is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get the current playback speed.
    pub fn speed(&self) -> f64 {
        self.speed
    }

    /// Set the playback speed.
    ///
    /// A speed of 1.0 is normal, 2.0 is twice as fast, 0.5 is half speed.
    pub fn set_speed(&mut self, speed: f64) {
        // Clamp to reasonable values
        self.speed = speed.clamp(0.1, 10.0);
    }

    /// Start or resume the clock.
    pub fn play(&mut self) {
        if !self.running {
            self.running = true;
            self.last_update = Some(Instant::now());
        }
    }

    /// Pause the clock.
    pub fn pause(&mut self) {
        if self.running {
            // Update position before pausing
            self.update();
            self.running = false;
            self.last_update = None;
        }
    }

    /// Toggle play/pause.
    pub fn toggle(&mut self) {
        if self.running {
            self.pause();
        } else {
            self.play();
        }
    }

    /// Seek to a specific position.
    pub fn seek(&mut self, position: Timestamp) {
        self.position = position.clamp(Timestamp::zero(), self.duration);
        self.last_update = if self.running {
            Some(Instant::now())
        } else {
            None
        };
    }

    /// Seek to the beginning.
    pub fn seek_to_start(&mut self) {
        self.seek(Timestamp::zero());
    }

    /// Seek to the end.
    pub fn seek_to_end(&mut self) {
        self.seek(self.duration);
    }

    /// Seek by a relative amount.
    pub fn seek_by(&mut self, delta_micros: i64) {
        let current = self.position.as_micros() as i64;
        let new_pos = (current + delta_micros).max(0) as u64;
        self.seek(Timestamp::from_micros(new_pos));
    }

    /// Step forward by one frame (at 60fps).
    pub fn step_forward(&mut self) {
        self.seek_by(16_667); // ~16.67ms for 60fps
    }

    /// Step backward by one frame (at 60fps).
    pub fn step_backward(&mut self) {
        self.seek_by(-16_667);
    }

    /// Update the clock based on elapsed real time.
    ///
    /// Call this every frame to advance the clock.
    /// Returns true if the clock reached the end.
    pub fn update(&mut self) -> bool {
        if !self.running {
            return false;
        }

        if let Some(last) = self.last_update {
            let elapsed = last.elapsed();
            let scaled_elapsed = Duration::from_secs_f64(elapsed.as_secs_f64() * self.speed);
            let scaled_micros = scaled_elapsed.as_micros() as u64;

            let new_pos = self.position.as_micros().saturating_add(scaled_micros);
            self.position = Timestamp::from_micros(new_pos);

            // Check if we've reached the end
            if self.position >= self.duration {
                self.position = self.duration;
                self.running = false;
                self.last_update = None;
                return true;
            }

            self.last_update = Some(Instant::now());
        }

        false
    }

    /// Get the progress as a percentage (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        if self.duration.as_micros() == 0 {
            return 0.0;
        }
        (self.position.as_micros() as f32) / (self.duration.as_micros() as f32)
    }

    /// Check if the clock has reached the end.
    pub fn is_at_end(&self) -> bool {
        self.position >= self.duration
    }

    /// Check if the clock is at the beginning.
    pub fn is_at_start(&self) -> bool {
        self.position.as_micros() == 0
    }

    /// Reset the clock to the beginning and stop.
    pub fn reset(&mut self) {
        self.position = Timestamp::zero();
        self.running = false;
        self.last_update = None;
    }
}

impl Default for VirtualClock {
    fn default() -> Self {
        Self::new(Timestamp::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_creation() {
        let clock = VirtualClock::new(Timestamp::from_micros(1_000_000)); // 1 second
        assert!(!clock.is_running());
        assert_eq!(clock.position().as_micros(), 0);
        assert_eq!(clock.speed(), 1.0);
    }

    #[test]
    fn test_seek() {
        let mut clock = VirtualClock::new(Timestamp::from_micros(1_000_000));

        clock.seek(Timestamp::from_micros(500_000));
        assert_eq!(clock.position().as_micros(), 500_000);

        // Seek beyond duration should clamp
        clock.seek(Timestamp::from_micros(2_000_000));
        assert_eq!(clock.position().as_micros(), 1_000_000);

        clock.seek_to_start();
        assert_eq!(clock.position().as_micros(), 0);
    }

    #[test]
    fn test_seek_by() {
        let mut clock = VirtualClock::new(Timestamp::from_micros(1_000_000));

        clock.seek(Timestamp::from_micros(500_000));
        clock.seek_by(100_000);
        assert_eq!(clock.position().as_micros(), 600_000);

        clock.seek_by(-200_000);
        assert_eq!(clock.position().as_micros(), 400_000);

        // Seek before start should clamp to 0
        clock.seek_by(-1_000_000);
        assert_eq!(clock.position().as_micros(), 0);
    }

    #[test]
    fn test_progress() {
        let mut clock = VirtualClock::new(Timestamp::from_micros(1_000_000));
        assert_eq!(clock.progress(), 0.0);

        clock.seek(Timestamp::from_micros(500_000));
        assert!((clock.progress() - 0.5).abs() < 0.001);

        clock.seek(Timestamp::from_micros(1_000_000));
        assert!((clock.progress() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_speed() {
        let mut clock = VirtualClock::new(Timestamp::from_micros(1_000_000));

        clock.set_speed(2.0);
        assert_eq!(clock.speed(), 2.0);

        // Speed should be clamped
        clock.set_speed(100.0);
        assert_eq!(clock.speed(), 10.0);

        clock.set_speed(0.01);
        assert_eq!(clock.speed(), 0.1);
    }

    #[test]
    fn test_play_pause_toggle() {
        let mut clock = VirtualClock::new(Timestamp::from_micros(1_000_000));
        assert!(!clock.is_running());

        clock.play();
        assert!(clock.is_running());

        clock.pause();
        assert!(!clock.is_running());

        clock.toggle();
        assert!(clock.is_running());

        clock.toggle();
        assert!(!clock.is_running());
    }
}
