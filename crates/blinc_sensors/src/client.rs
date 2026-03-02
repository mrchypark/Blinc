use crate::{SensorBackend, SensorConfig, SensorError, SensorFrame, SensorKind, SensorStatus};

/// High-level typed client for sensor operations.
pub struct SensorClient<B: SensorBackend> {
    backend: B,
}

impl<B: SensorBackend> SensorClient<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn configure(&self, config: &SensorConfig) -> Result<(), SensorError> {
        if config.imu_hz == 0 {
            return Err(SensorError::InvalidConfig(
                "imu_hz must be greater than 0".to_string(),
            ));
        }
        if config.gps_hz == 0 {
            return Err(SensorError::InvalidConfig(
                "gps_hz must be greater than 0".to_string(),
            ));
        }
        self.backend.configure(config)
    }

    pub fn start_session(&self, session_id: &str) -> Result<(), SensorError> {
        if session_id.trim().is_empty() {
            return Err(SensorError::InvalidSessionId);
        }
        self.backend.start(session_id)
    }

    pub fn stop_session(&self, session_id: &str) -> Result<(), SensorError> {
        if session_id.trim().is_empty() {
            return Err(SensorError::InvalidSessionId);
        }
        self.backend.stop(session_id)
    }

    pub fn status(&self) -> Result<SensorStatus, SensorError> {
        self.backend.status()
    }

    pub fn drain_frames(&self, max_frames: usize) -> Result<Vec<SensorFrame>, SensorError> {
        self.backend.drain_frames(max_frames)
    }

    pub fn supported_kinds(&self) -> Result<Vec<SensorKind>, SensorError> {
        self.backend.supported_kinds()
    }
}
