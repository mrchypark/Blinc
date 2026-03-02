//! Unified sensor API for Blinc mobile platforms.
//!
//! This crate provides a platform-agnostic sensor API on top of Blinc's native
//! bridge (`blinc_core::native_bridge`) with a layered structure inspired by
//! Drift-style platform services:
//! - `types`: shared domain model
//! - `backend`: backend abstraction
//! - `permissions`: sensor-related permission facade
//! - `runtime`: cross-platform session lifecycle and polling controller

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

/// Backward-compatible module path: `blinc_sensors::native_bridge::NativeBridgeBackend`.
pub mod native_bridge {
    pub use crate::bridge_backend::NativeBridgeBackend;
}
