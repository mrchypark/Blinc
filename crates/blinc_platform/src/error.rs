//! Platform error types

use thiserror::Error;

/// Platform-related errors
#[derive(Error, Debug)]
pub enum PlatformError {
    /// Failed to initialize platform
    #[error("Platform initialization failed: {0}")]
    InitFailed(String),

    /// Failed to create event loop
    #[error("Failed to create event loop: {0}")]
    EventLoop(String),

    /// Failed to create window
    #[error("Failed to create window: {0}")]
    WindowCreation(String),

    /// Platform not available
    #[error("Platform not available: {0}")]
    Unavailable(String),

    /// Platform not supported on this OS
    #[error("Platform not supported: {0}")]
    Unsupported(String),

    /// Failed to create WebView
    #[error("WebView creation failed: {0}")]
    WebViewCreation(String),

    /// WebView operation failed
    #[error("WebView operation failed: {0}")]
    WebViewOperation(String),

    /// Generic platform error
    #[error("Platform error: {0}")]
    Other(String),

    /// Failed to load asset
    #[error("Asset load failed: {0}")]
    AssetLoad(String),
}

/// Result type for platform operations
pub type Result<T> = std::result::Result<T, PlatformError>;
