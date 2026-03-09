use blinc_core::native_bridge::{native_call, NativeBridgeError, NativeValue};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Granted,
    Denied,
    PermanentlyDenied,
    Restricted,
    Limited,
    NotDetermined,
    Provisional,
    Pending,
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

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "granted" => Some(Self::Granted),
            "denied" => Some(Self::Denied),
            "permanently_denied" => Some(Self::PermanentlyDenied),
            "restricted" => Some(Self::Restricted),
            "limited" => Some(Self::Limited),
            "not_determined" => Some(Self::NotDetermined),
            "provisional" => Some(Self::Provisional),
            "pending" => Some(Self::Pending),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// Structured capability snapshot for a permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionCapability {
    pub status: PermissionStatus,
    #[serde(default)]
    pub can_request: bool,
    #[serde(default)]
    pub requires_settings_redirect: bool,
    #[serde(default = "default_supported")]
    pub supported: bool,
}

/// Structured result from a permission request round-trip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRequestResult {
    pub status: PermissionStatus,
    #[serde(default)]
    pub previous_status: Option<PermissionStatus>,
    #[serde(default = "default_can_request_again")]
    pub can_request_again: bool,
    #[serde(default)]
    pub requires_settings_redirect: bool,
}

fn default_supported() -> bool {
    true
}

fn default_can_request_again() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct PermissionCapabilityPayload {
    status: PermissionStatusWire,
    #[serde(default, alias = "canRequest")]
    can_request: bool,
    #[serde(default, alias = "requiresSettingsRedirect")]
    requires_settings_redirect: bool,
    #[serde(default = "default_supported")]
    supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct PermissionRequestPayload {
    status: PermissionStatusWire,
    #[serde(default, alias = "previousStatus")]
    previous_status: Option<PermissionStatusWire>,
    #[serde(default = "default_can_request_again", alias = "canRequestAgain")]
    can_request_again: bool,
    #[serde(default, alias = "requiresSettingsRedirect")]
    requires_settings_redirect: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum PermissionStatusWire {
    Named(String),
    Bool(bool),
}

impl PermissionStatusWire {
    fn into_status(self) -> Result<PermissionStatus, PlatformError> {
        match self {
            Self::Named(name) => PermissionStatus::from_str(&name).ok_or_else(|| {
                PlatformError::Other(format!("Unknown permission status payload: {name}"))
            }),
            Self::Bool(granted) => Ok(PermissionStatus::from_bool(granted)),
        }
    }
}

impl PermissionCapabilityPayload {
    fn into_capability(self) -> Result<PermissionCapability, PlatformError> {
        Ok(PermissionCapability {
            status: self.status.into_status()?,
            can_request: self.can_request,
            requires_settings_redirect: self.requires_settings_redirect,
            supported: self.supported,
        })
    }
}

impl PermissionRequestPayload {
    fn into_result(self) -> Result<PermissionRequestResult, PlatformError> {
        Ok(PermissionRequestResult {
            status: self.status.into_status()?,
            previous_status: self
                .previous_status
                .map(PermissionStatusWire::into_status)
                .transpose()?,
            can_request_again: self.can_request_again,
            requires_settings_redirect: self.requires_settings_redirect,
        })
    }
}

fn decode_json_payload<T>(value: NativeValue) -> Result<T, PlatformError>
where
    T: for<'de> Deserialize<'de>,
{
    match value {
        NativeValue::Json(json) => Ok(serde_json::from_str(&json)?),
        NativeValue::String(json) => Ok(serde_json::from_str(&json)?),
        other => Err(PlatformError::Bridge(NativeBridgeError::TypeMismatch {
            expected: "Json",
            actual: other.type_name().to_string(),
        })),
    }
}

fn fallback_capability(status: PermissionStatus) -> PermissionCapability {
    PermissionCapability {
        can_request: matches!(
            status,
            PermissionStatus::Denied | PermissionStatus::NotDetermined
        ),
        requires_settings_redirect: matches!(
            status,
            PermissionStatus::PermanentlyDenied | PermissionStatus::Restricted
        ),
        supported: !matches!(status, PermissionStatus::Unknown),
        status,
    }
}

/// Check the current status of a runtime permission.
pub fn status(kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
    Ok(capability(kind)?.status)
}

/// Fetch the current permission capability snapshot.
pub fn capability(kind: PermissionKind) -> Result<PermissionCapability, PlatformError> {
    match native_call::<NativeValue, _>("permissions", kind.has_name(), ()) {
        Ok(NativeValue::Bool(granted)) => {
            Ok(fallback_capability(PermissionStatus::from_bool(granted)))
        }
        Ok(other @ (NativeValue::Json(_) | NativeValue::String(_))) => {
            decode_json_payload::<PermissionCapabilityPayload>(other)?.into_capability()
        }
        Ok(other) => Err(PlatformError::Bridge(NativeBridgeError::TypeMismatch {
            expected: "Bool|Json",
            actual: other.type_name().to_string(),
        })),
        Err(NativeBridgeError::NotRegistered { .. }) => {
            Ok(fallback_capability(PermissionStatus::Unknown))
        }
        Err(err) => Err(PlatformError::Bridge(err)),
    }
}

/// Request a runtime permission from the OS.
pub fn request(kind: PermissionKind) -> Result<PermissionRequestResult, PlatformError> {
    match native_call::<NativeValue, _>("permissions", kind.request_name(), ())? {
        NativeValue::Bool(granted) => Ok(PermissionRequestResult {
            status: PermissionStatus::from_bool(granted),
            previous_status: None,
            can_request_again: !granted,
            requires_settings_redirect: false,
        }),
        other @ (NativeValue::Json(_) | NativeValue::String(_)) => {
            decode_json_payload::<PermissionRequestPayload>(other)?.into_result()
        }
        other => Err(PlatformError::Bridge(NativeBridgeError::TypeMismatch {
            expected: "Bool|Json",
            actual: other.type_name().to_string(),
        })),
    }
}

/// Open the OS settings page for the current application.
pub fn open_settings() -> Result<bool, PlatformError> {
    Ok(native_call("permissions", "open_settings", ())?)
}

/// Convenience helper returning whether permission is currently granted.
pub fn is_granted(kind: PermissionKind) -> Result<bool, PlatformError> {
    Ok(matches!(status(kind)?, PermissionStatus::Granted))
}
