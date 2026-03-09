use crate::error::PlatformError;
use crate::permissions::{self, PermissionKind, PermissionRequestResult, PermissionStatus};

/// Check current microphone permission status.
pub fn status() -> Result<PermissionStatus, PlatformError> {
    permissions::status(PermissionKind::Microphone)
}

/// Request microphone permission.
pub fn request() -> Result<PermissionRequestResult, PlatformError> {
    permissions::request(PermissionKind::Microphone)
}
