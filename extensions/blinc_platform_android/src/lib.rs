//! Blinc Android Platform
//!
//! Native Activity integration and JNI bridge.

pub mod activity;
pub mod input;

// Android-specific entry point
#[cfg(target_os = "android")]
pub use activity::android_main;
