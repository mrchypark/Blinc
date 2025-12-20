//! Blinc Animation System
//!
//! Spring physics, keyframe animations, and timeline orchestration.
//!
//! # Features
//!
//! - **Spring Physics**: RK4-integrated springs with stiffness, damping, mass
//! - **Keyframe Animations**: Timed sequences with easing functions
//! - **Timelines**: Orchestrate multiple animations with offsets
//! - **Interruptible**: Animations inherit velocity when interrupted

pub mod easing;
pub mod keyframe;
pub mod scheduler;
pub mod spring;
pub mod timeline;

pub use easing::Easing;
pub use keyframe::{Keyframe, KeyframeAnimation};
pub use scheduler::AnimationScheduler;
pub use spring::{Spring, SpringConfig};
pub use timeline::Timeline;
