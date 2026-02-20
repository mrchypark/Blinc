use std::sync::Mutex;

use blinc_sensors::{
    SensorAccuracy, SensorBackend, SensorClient, SensorConfig, SensorError, SensorFrame,
    SensorKind, SensorStatus,
};

#[derive(Default)]
struct MockBackend {
    calls: Mutex<Vec<String>>,
}

impl SensorBackend for MockBackend {
    fn configure(&self, _config: &SensorConfig) -> Result<(), SensorError> {
        self.calls.lock().unwrap().push("configure".to_string());
        Ok(())
    }

    fn start(&self, session_id: &str) -> Result<(), SensorError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("start:{session_id}"));
        Ok(())
    }

    fn stop(&self, session_id: &str) -> Result<(), SensorError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("stop:{session_id}"));
        Ok(())
    }

    fn status(&self) -> Result<SensorStatus, SensorError> {
        self.calls.lock().unwrap().push("status".to_string());
        Ok(SensorStatus::default())
    }

    fn drain_frames(&self, max_frames: usize) -> Result<Vec<SensorFrame>, SensorError> {
        self.calls
            .lock()
            .unwrap()
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
            .unwrap()
            .push("supported_kinds".to_string());
        Ok(vec![SensorKind::Gps, SensorKind::Accelerometer])
    }
}

#[test]
fn start_session_rejects_empty_id() {
    let client = SensorClient::new(MockBackend::default());
    let err = client.start_session("").unwrap_err();
    assert!(matches!(err, SensorError::InvalidSessionId));
}

#[test]
fn stop_session_rejects_empty_id() {
    let client = SensorClient::new(MockBackend::default());
    let err = client.stop_session("").unwrap_err();
    assert!(matches!(err, SensorError::InvalidSessionId));
}

#[test]
fn configure_rejects_zero_rates() {
    let client = SensorClient::new(MockBackend::default());

    let mut imu_zero = SensorConfig::default();
    imu_zero.imu_hz = 0;
    let imu_err = client.configure(&imu_zero).unwrap_err();
    assert!(matches!(imu_err, SensorError::InvalidConfig(_)));

    let mut gps_zero = SensorConfig::default();
    gps_zero.gps_hz = 0;
    let gps_err = client.configure(&gps_zero).unwrap_err();
    assert!(matches!(gps_err, SensorError::InvalidConfig(_)));
}

#[test]
fn drain_frames_passthrough() {
    let client = SensorClient::new(MockBackend::default());
    let frames = client.drain_frames(32).unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].sensor, SensorKind::Accelerometer);
}

#[test]
fn supported_kinds_passthrough() {
    let client = SensorClient::new(MockBackend::default());
    let kinds = client.supported_kinds().unwrap();
    assert_eq!(kinds, vec![SensorKind::Gps, SensorKind::Accelerometer]);
}
