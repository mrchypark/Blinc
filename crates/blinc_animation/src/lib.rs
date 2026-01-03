//! Blinc Animation System
//!
//! Spring physics, keyframe animations, and timeline orchestration.
//!
//! # Features
//!
//! - **Spring Physics**: RK4-integrated springs with stiffness, damping, mass
//! - **Keyframe Animations**: Timed sequences with easing functions
//! - **Multi-Property Keyframes**: Animate multiple properties simultaneously
//! - **Timelines**: Orchestrate multiple animations with offsets
//! - **Interruptible**: Animations inherit velocity when interrupted
//! - **Animation Presets**: Common entry/exit animations
//! - **AnimationContext**: Platform-agnostic animation management trait

pub mod context;
pub mod easing;
pub mod keyframe;
pub mod presets;
pub mod scheduler;
pub mod spring;
pub mod timeline;

pub use context::{
    AnimationContext, AnimationContextExt, SharedAnimatedTimeline, SharedAnimatedValue,
};
pub use easing::Easing;
pub use keyframe::{
    FillMode, Keyframe, KeyframeAnimation, KeyframeProperties, MultiKeyframe,
    MultiKeyframeAnimation, PlayDirection,
};
pub use presets::AnimationPreset;
pub use scheduler::{
    AnimatedKeyframe, AnimatedTimeline, AnimatedValue, AnimationScheduler, KeyframeId,
    SchedulerHandle, SpringId, TimelineId,
};
pub use spring::{Spring, SpringConfig};
pub use timeline::{Timeline, TimelineEntryId};
