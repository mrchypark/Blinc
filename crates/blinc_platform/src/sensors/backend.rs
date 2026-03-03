use crate::{SensorConfig, SensorError, SensorFrame, SensorKind, SensorStatus};

/// Pluggable backend abstraction for sensor operations.
pub trait SensorBackend: Send + Sync {
    fn configure(&self, config: &SensorConfig) -> Result<(), SensorError>;
    fn start(&self, session_id: &str) -> Result<(), SensorError>;
    fn stop(&self, session_id: &str) -> Result<(), SensorError>;
    fn status(&self) -> Result<SensorStatus, SensorError>;
    fn drain_frames(&self, max_frames: usize) -> Result<Vec<SensorFrame>, SensorError>;

    fn supported_kinds(&self) -> Result<Vec<SensorKind>, SensorError> {
        Err(SensorError::Backend(
            "supported_kinds is not implemented for this backend".to_string(),
        ))
    }
}
