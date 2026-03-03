use blinc_core::native_bridge::NativeBridgeError;
use thiserror::Error;

use crate::SensorPermissionState;

/// Errors returned by [`crate::SensorClient`] and backends.
#[derive(Debug, Error)]
pub enum SensorError {
    #[error("session id must not be empty")]
    InvalidSessionId,

    #[error("invalid config: {0}")]
    InvalidConfig(String),

    #[error("required sensor permissions are not ready (location={location}, motion={motion})")]
    RequiredPermissionsNotReady { location: bool, motion: bool },

    #[error("native bridge error: {0}")]
    Bridge(#[from] NativeBridgeError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("backend error: {0}")]
    Backend(String),
}

impl SensorError {
    pub fn required_permissions_not_ready(state: SensorPermissionState) -> Self {
        Self::RequiredPermissionsNotReady {
            location: state.has_location,
            motion: state.has_motion,
        }
    }
}
