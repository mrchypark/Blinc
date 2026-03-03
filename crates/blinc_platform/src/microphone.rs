use crate::error::PlatformError;
use crate::permissions::{self, PermissionKind, PermissionStatus};

/// Check current microphone permission status.
pub fn status() -> Result<PermissionStatus, PlatformError> {
    permissions::status(PermissionKind::Microphone)
}

/// Request microphone permission.
pub fn request() -> Result<PermissionStatus, PlatformError> {
    permissions::request(PermissionKind::Microphone)
}
