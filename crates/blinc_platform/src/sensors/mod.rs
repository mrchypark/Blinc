//! Unified sensor API for Blinc mobile platforms.
//!
//! This module is migrated from the former `blinc_sensors` crate and provides
//! a platform-agnostic sensor API on top of Blinc's native bridge.

mod backend;
mod bridge_backend;
mod client;
mod error;
mod permissions;
mod runtime;
mod types;

pub use backend::SensorBackend;
pub use client::SensorClient;
pub use error::SensorError;
pub use permissions::{
    NativeBridgePermissionBackend, SensorPermissionBackend, SensorPermissionService,
    SensorPermissionState,
};
pub use runtime::{SensorBatchSummary, SensorProbeState, SensorRuntimeController};
pub use types::{SensorAccuracy, SensorConfig, SensorFrame, SensorKind, SensorStatus};

/// Backward-compatible module path for native bridge backend.
pub mod native_bridge {
    pub use super::bridge_backend::NativeBridgeBackend;
}
