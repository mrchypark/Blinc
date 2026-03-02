use serde::{Deserialize, Serialize};

/// Sensor type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorKind {
    Gps,
    Accelerometer,
    LinearAcceleration,
    Gravity,
    Gyroscope,
    RotationVector,
    Quaternion,
    Magnetometer,
    Barometer,
    AmbientLight,
    Proximity,
    AmbientTemperature,
    RelativeHumidity,
    StepCounter,
    StepDetector,
    SignificantMotion,
    Heading,
    DeviceMotion,
    Cadence,
    FloorClimb,
    Activity,
    HeartRate,
}

impl SensorKind {
    /// Returns the canonical snake_case name used for native bridge JSON.
    pub const fn as_str(self) -> &'static str {
        match self {
            SensorKind::Gps => "gps",
            SensorKind::Accelerometer => "accelerometer",
            SensorKind::LinearAcceleration => "linear_acceleration",
            SensorKind::Gravity => "gravity",
            SensorKind::Gyroscope => "gyroscope",
            SensorKind::RotationVector => "rotation_vector",
            SensorKind::Quaternion => "quaternion",
            SensorKind::Magnetometer => "magnetometer",
            SensorKind::Barometer => "barometer",
            SensorKind::AmbientLight => "ambient_light",
            SensorKind::Proximity => "proximity",
            SensorKind::AmbientTemperature => "ambient_temperature",
            SensorKind::RelativeHumidity => "relative_humidity",
            SensorKind::StepCounter => "step_counter",
            SensorKind::StepDetector => "step_detector",
            SensorKind::SignificantMotion => "significant_motion",
            SensorKind::Heading => "heading",
            SensorKind::DeviceMotion => "device_motion",
            SensorKind::Cadence => "cadence",
            SensorKind::FloorClimb => "floor_climb",
            SensorKind::Activity => "activity",
            SensorKind::HeartRate => "heart_rate",
        }
    }

    pub fn known_kinds() -> &'static [SensorKind] {
        const KINDS: &[SensorKind] = &[
            SensorKind::Gps,
            SensorKind::Accelerometer,
            SensorKind::LinearAcceleration,
            SensorKind::Gravity,
            SensorKind::Gyroscope,
            SensorKind::RotationVector,
            SensorKind::Quaternion,
            SensorKind::Magnetometer,
            SensorKind::Barometer,
            SensorKind::AmbientLight,
            SensorKind::Proximity,
            SensorKind::AmbientTemperature,
            SensorKind::RelativeHumidity,
            SensorKind::StepCounter,
            SensorKind::StepDetector,
            SensorKind::SignificantMotion,
            SensorKind::Heading,
            SensorKind::DeviceMotion,
            SensorKind::Cadence,
            SensorKind::FloorClimb,
            SensorKind::Activity,
            SensorKind::HeartRate,
        ];
        KINDS
    }
}

impl std::fmt::Display for SensorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reported quality level from native sensor frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorAccuracy {
    Unreliable,
    Low,
    Medium,
    High,
}

/// A normalized sensor frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorFrame {
    pub seq: u64,
    pub sensor: SensorKind,
    pub time_monotonic_ns: u64,
    pub time_unix_ms: i64,
    pub accuracy: SensorAccuracy,
    pub values: Vec<f32>,
}

/// Runtime sensor configuration shared by Android/iOS backends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    /// Sensor streams that should be enabled.
    pub enabled: Vec<SensorKind>,
    /// Desired GPS sampling rate.
    pub gps_hz: u16,
    /// Desired IMU sampling rate.
    pub imu_hz: u16,
    /// Native flush interval for batched frames.
    pub frame_flush_ms: u16,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                SensorKind::Gps,
                SensorKind::Accelerometer,
                SensorKind::Gyroscope,
            ],
            gps_hz: 1,
            imu_hz: 50,
            frame_flush_ms: 200,
        }
    }
}

/// Snapshot of current backend state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensorStatus {
    pub running: bool,
    pub buffered_frames: usize,
    pub active_session_id: Option<String>,
}

impl Default for SensorStatus {
    fn default() -> Self {
        Self {
            running: false,
            buffered_frames: 0,
            active_session_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SensorKind;

    #[test]
    fn sensor_kind_as_str_matches_serde_snake_case() {
        for kind in SensorKind::known_kinds() {
            let serde = serde_json::to_string(kind).expect("serde_json::to_string");
            let serde = serde.trim_matches('"');
            assert_eq!(kind.as_str(), serde);
        }
    }
}
