//! Replay engine for recorded sessions.
//!
//! This module provides:
//! - `VirtualClock` - Deterministic time control for replay
//! - `EventSimulator` - Inject recorded events into the UI
//! - `ReplayPlayer` - Play back recorded sessions with time control
//!
//! # Example
//!
//! ```ignore
//! use blinc_recorder::replay::{ReplayPlayer, ReplayConfig};
//!
//! let export = session.export();
//! let mut player = ReplayPlayer::new(export, ReplayConfig::default());
//!
//! // Play at 2x speed
//! player.set_playback_speed(2.0);
//! player.play();
//!
//! // Or step through frame by frame
//! while player.has_next() {
//!     let frame_events = player.step();
//!     for event in frame_events {
//!         // Process event...
//!     }
//! }
//! ```

mod clock;
mod player;
mod simulator;

pub use clock::VirtualClock;
pub use player::{FrameUpdate, ReplayConfig, ReplayPlayer, ReplayState};
pub use simulator::{EventSimulator, SimulatedInput};
