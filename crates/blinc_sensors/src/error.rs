use blinc_core::native_bridge::NativeBridgeError;
use thiserror::Error;

/// Errors returned by [`crate::SensorClient`] and backends.
#[derive(Debug, Error)]
pub enum SensorError {
    #[error("session id must not be empty")]
    InvalidSessionId,

    #[error("invalid config: {0}")]
    InvalidConfig(String),

    #[error("native bridge error: {0}")]
    Bridge(#[from] NativeBridgeError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("backend error: {0}")]
    Backend(String),
}
