use blinc_core::native_bridge::{native_call, NativeBridgeError};

use crate::error::PlatformError;

/// Kind of runtime permission exposed by the platform layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionKind {
    Microphone,
    LocationWhenInUse,
    LocationAlways,
    Motion,
    BluetoothScan,
    BluetoothConnect,
    Camera,
    Photos,
    Notifications,
}

impl PermissionKind {
    fn has_name(self) -> &'static str {
        match self {
            PermissionKind::Microphone => "has_microphone",
            PermissionKind::LocationWhenInUse => "has_location",
            PermissionKind::LocationAlways => "has_location_always",
            PermissionKind::Motion => "has_motion",
            PermissionKind::BluetoothScan => "has_bluetooth_scan",
            PermissionKind::BluetoothConnect => "has_bluetooth_connect",
            PermissionKind::Camera => "has_camera",
            PermissionKind::Photos => "has_photos",
            PermissionKind::Notifications => "has_notifications",
        }
    }

    fn request_name(self) -> &'static str {
        match self {
            PermissionKind::Microphone => "request_microphone",
            PermissionKind::LocationWhenInUse => "request_location_when_in_use",
            PermissionKind::LocationAlways => "request_location_always",
            PermissionKind::Motion => "request_motion",
            PermissionKind::BluetoothScan => "request_bluetooth_scan",
            PermissionKind::BluetoothConnect => "request_bluetooth_connect",
            PermissionKind::Camera => "request_camera",
            PermissionKind::Photos => "request_photos",
            PermissionKind::Notifications => "request_notifications",
        }
    }
}

/// Unified runtime permission status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionStatus {
    Granted,
    Denied,
    PermanentlyDenied,
    Restricted,
    Limited,
    NotDetermined,
    Provisional,
    Unknown,
}

impl PermissionStatus {
    fn from_bool(granted: bool) -> Self {
        if granted {
            Self::Granted
        } else {
            Self::Denied
        }
    }
}

/// Check the current status of a runtime permission.
pub fn status(kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
    match native_call::<bool, _>("permissions", kind.has_name(), ()) {
        Ok(granted) => Ok(PermissionStatus::from_bool(granted)),
        Err(NativeBridgeError::NotRegistered { .. }) => Ok(PermissionStatus::Unknown),
        Err(err) => Err(PlatformError::Bridge(err)),
    }
}

/// Request a runtime permission from the OS.
pub fn request(kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
    let granted: bool = native_call("permissions", kind.request_name(), ())?;
    Ok(PermissionStatus::from_bool(granted))
}

/// Convenience helper returning whether permission is currently granted.
pub fn is_granted(kind: PermissionKind) -> Result<bool, PlatformError> {
    Ok(matches!(status(kind)?, PermissionStatus::Granted))
}
