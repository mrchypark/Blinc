use crate::permissions::{self, PermissionKind, PermissionStatus};
use crate::SensorError;

/// Snapshot of sensor-related permission states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorPermissionState {
    pub has_location: bool,
    pub has_motion: bool,
}

impl SensorPermissionState {
    pub fn ready(self) -> bool {
        self.has_location && self.has_motion
    }
}

/// Permission backend abstraction for mobile sensor access.
pub trait SensorPermissionBackend: Send + Sync {
    fn has_location(&self) -> Result<bool, SensorError>;
    fn has_motion(&self) -> Result<bool, SensorError>;
    fn request_location_when_in_use(&self) -> Result<bool, SensorError>;
    fn request_motion(&self) -> Result<bool, SensorError>;
}

/// Native bridge permission backend using `permissions.*` handlers.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeBridgePermissionBackend;

impl SensorPermissionBackend for NativeBridgePermissionBackend {
    fn has_location(&self) -> Result<bool, SensorError> {
        permissions::is_granted(PermissionKind::LocationWhenInUse).map_err(|err| match err {
            crate::PlatformError::Bridge(source) => SensorError::Bridge(source),
            crate::PlatformError::Serialization(source) => SensorError::Serialization(source),
            other => SensorError::Backend(other.to_string()),
        })
    }

    fn has_motion(&self) -> Result<bool, SensorError> {
        permissions::is_granted(PermissionKind::Motion).map_err(|err| match err {
            crate::PlatformError::Bridge(source) => SensorError::Bridge(source),
            crate::PlatformError::Serialization(source) => SensorError::Serialization(source),
            other => SensorError::Backend(other.to_string()),
        })
    }

    fn request_location_when_in_use(&self) -> Result<bool, SensorError> {
        permissions::request(PermissionKind::LocationWhenInUse)
            .map(|result| result.status == PermissionStatus::Granted)
            .map_err(|err| match err {
                crate::PlatformError::Bridge(source) => SensorError::Bridge(source),
                crate::PlatformError::Serialization(source) => SensorError::Serialization(source),
                other => SensorError::Backend(other.to_string()),
            })
    }

    fn request_motion(&self) -> Result<bool, SensorError> {
        permissions::request(PermissionKind::Motion)
            .map(|result| result.status == PermissionStatus::Granted)
            .map_err(|err| match err {
                crate::PlatformError::Bridge(source) => SensorError::Bridge(source),
                crate::PlatformError::Serialization(source) => SensorError::Serialization(source),
                other => SensorError::Backend(other.to_string()),
            })
    }
}

/// Permission facade used by sensor runtime control.
pub struct SensorPermissionService<P: SensorPermissionBackend> {
    backend: P,
}

impl<P: SensorPermissionBackend> SensorPermissionService<P> {
    pub fn new(backend: P) -> Self {
        Self { backend }
    }

    pub fn state(&self) -> Result<SensorPermissionState, SensorError> {
        Ok(SensorPermissionState {
            has_location: self.backend.has_location()?,
            has_motion: self.backend.has_motion()?,
        })
    }

    /// Best-effort permission request for location + motion access.
    pub fn request_required_permissions(&self) -> Result<SensorPermissionState, SensorError> {
        let initial_state = self.state()?;
        if !initial_state.has_location {
            let _ = self.backend.request_location_when_in_use()?;
        }
        if !initial_state.has_motion {
            let _ = self.backend.request_motion()?;
        }
        self.state()
    }
}
