//! Blinc iOS Platform
//!
//! UIKit integration and Metal rendering.

pub mod app;
pub mod input;

// iOS-specific entry point
#[cfg(target_os = "ios")]
pub use app::ios_main;
