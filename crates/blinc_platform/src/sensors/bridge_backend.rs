use blinc_core::native_bridge::native_call;

use crate::{SensorBackend, SensorConfig, SensorError, SensorFrame, SensorKind, SensorStatus};

/// Native bridge backend.
///
/// Expects native handlers:
/// - `sensor.configure(config_json: String) -> Bool`
/// - `sensor.start(session_id: String) -> Bool`
/// - `sensor.stop(session_id: String) -> Bool`
/// - `sensor.status() -> String` (JSON object)
/// - `sensor.drain_frames(max_frames: Int32) -> String` (JSON array)
/// - `sensor.supported_kinds() -> String` (JSON array)
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeBridgeBackend;

impl SensorBackend for NativeBridgeBackend {
    fn configure(&self, config: &SensorConfig) -> Result<(), SensorError> {
        let json = serde_json::to_string(config)?;
        let ok: bool = native_call("sensor", "configure", (json,))?;
        if ok {
            Ok(())
        } else {
            Err(SensorError::Backend(
                "native sensor.configure returned false".to_string(),
            ))
        }
    }

    fn start(&self, session_id: &str) -> Result<(), SensorError> {
        let ok: bool = native_call("sensor", "start", (session_id.to_string(),))?;
        if ok {
            Ok(())
        } else {
            Err(SensorError::Backend(
                "native sensor.start returned false".to_string(),
            ))
        }
    }

    fn stop(&self, session_id: &str) -> Result<(), SensorError> {
        let ok: bool = native_call("sensor", "stop", (session_id.to_string(),))?;
        if ok {
            Ok(())
        } else {
            Err(SensorError::Backend(
                "native sensor.stop returned false".to_string(),
            ))
        }
    }

    fn status(&self) -> Result<SensorStatus, SensorError> {
        let json: String = native_call("sensor", "status", ())?;
        Ok(serde_json::from_str(&json)?)
    }

    fn drain_frames(&self, max_frames: usize) -> Result<Vec<SensorFrame>, SensorError> {
        let max_frames_i32 = i32::try_from(max_frames)
            .map_err(|_| SensorError::InvalidConfig("max_frames exceeds i32 range".to_string()))?;
        let json: String = native_call("sensor", "drain_frames", (max_frames_i32,))?;
        Ok(serde_json::from_str(&json)?)
    }

    fn supported_kinds(&self) -> Result<Vec<SensorKind>, SensorError> {
        let json: String = native_call("sensor", "supported_kinds", ())?;
        Ok(serde_json::from_str(&json)?)
    }
}
