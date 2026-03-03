use std::sync::Mutex;

use blinc_platform::sensors::{
    SensorAccuracy, SensorBackend, SensorClient, SensorConfig, SensorError, SensorFrame,
    SensorKind, SensorStatus,
};

#[derive(Default)]
struct MockBackend {
    calls: Mutex<Vec<String>>,
}

impl SensorBackend for MockBackend {
    fn configure(&self, _config: &SensorConfig) -> Result<(), SensorError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push("configure".to_string());
        Ok(())
    }

    fn start(&self, session_id: &str) -> Result<(), SensorError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(format!("start:{session_id}"));
        Ok(())
    }

    fn stop(&self, session_id: &str) -> Result<(), SensorError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(format!("stop:{session_id}"));
        Ok(())
    }

    fn status(&self) -> Result<SensorStatus, SensorError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push("status".to_string());
        Ok(SensorStatus::default())
    }

    fn drain_frames(&self, max_frames: usize) -> Result<Vec<SensorFrame>, SensorError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(format!("drain:{max_frames}"));
        Ok(vec![SensorFrame {
            seq: 1,
            sensor: SensorKind::Accelerometer,
            time_monotonic_ns: 100,
            time_unix_ms: 1_700_000_000_000,
            accuracy: SensorAccuracy::High,
            values: vec![0.1, 0.2, 0.3],
        }])
    }

    fn supported_kinds(&self) -> Result<Vec<SensorKind>, SensorError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push("supported_kinds".to_string());
        Ok(vec![SensorKind::Gps, SensorKind::Accelerometer])
    }
}

#[test]
fn migrated_sensor_client_validates_and_passthroughs() {
    let client = SensorClient::new(MockBackend::default());

    let err = client
        .start_session("")
        .expect_err("empty session id must fail");
    assert!(matches!(err, SensorError::InvalidSessionId));

    let mut bad = SensorConfig::default();
    bad.imu_hz = 0;
    let err = client.configure(&bad).expect_err("zero imu must fail");
    assert!(matches!(err, SensorError::InvalidConfig(_)));

    let frames = client.drain_frames(32).expect("drain frames");
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].sensor, SensorKind::Accelerometer);

    let kinds = client.supported_kinds().expect("supported kinds");
    assert_eq!(kinds, vec![SensorKind::Gps, SensorKind::Accelerometer]);
}
