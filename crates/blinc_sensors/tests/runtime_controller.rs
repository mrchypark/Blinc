use std::sync::Mutex;

use blinc_sensors::{
    SensorAccuracy, SensorBackend, SensorClient, SensorConfig, SensorError, SensorFrame,
    SensorKind, SensorPermissionBackend, SensorPermissionService, SensorRuntimeController,
    SensorStatus,
};

#[derive(Default)]
struct MockBackend {
    running: Mutex<bool>,
}

impl SensorBackend for MockBackend {
    fn configure(&self, _config: &SensorConfig) -> Result<(), SensorError> {
        Ok(())
    }

    fn start(&self, _session_id: &str) -> Result<(), SensorError> {
        *self.running.lock().unwrap() = true;
        Ok(())
    }

    fn stop(&self, _session_id: &str) -> Result<(), SensorError> {
        *self.running.lock().unwrap() = false;
        Ok(())
    }

    fn status(&self) -> Result<SensorStatus, SensorError> {
        Ok(SensorStatus {
            running: *self.running.lock().unwrap(),
            buffered_frames: 0,
            active_session_id: Some("session".to_string()),
        })
    }

    fn drain_frames(&self, max_frames: usize) -> Result<Vec<SensorFrame>, SensorError> {
        if !*self.running.lock().unwrap() {
            return Ok(vec![]);
        }
        let n = max_frames.min(2);
        Ok((0..n)
            .map(|i| SensorFrame {
                seq: i as u64 + 1,
                sensor: if i % 2 == 0 {
                    SensorKind::Accelerometer
                } else {
                    SensorKind::Gyroscope
                },
                time_monotonic_ns: i as u64,
                time_unix_ms: i as i64,
                accuracy: SensorAccuracy::High,
                values: vec![i as f32, 0.0, 0.0],
            })
            .collect())
    }
}

#[derive(Default)]
struct MockPermissionBackend {
    has_location: Mutex<bool>,
    has_motion: Mutex<bool>,
}

impl SensorPermissionBackend for MockPermissionBackend {
    fn has_location(&self) -> Result<bool, SensorError> {
        Ok(*self.has_location.lock().unwrap())
    }

    fn has_motion(&self) -> Result<bool, SensorError> {
        Ok(*self.has_motion.lock().unwrap())
    }

    fn request_location_when_in_use(&self) -> Result<bool, SensorError> {
        *self.has_location.lock().unwrap() = true;
        Ok(true)
    }

    fn request_motion(&self) -> Result<bool, SensorError> {
        *self.has_motion.lock().unwrap() = true;
        Ok(true)
    }
}

#[test]
fn runtime_controller_starts_and_polls() {
    let client = SensorClient::new(MockBackend::default());
    let permissions = SensorPermissionService::new(MockPermissionBackend::default());
    let mut runtime = SensorRuntimeController::new(client, permissions, "runtime-test");
    runtime.configure(&SensorConfig::default()).unwrap();

    runtime.ensure_started().unwrap();
    assert!(runtime.running());

    let first = runtime.poll_batch(8, 1_000).unwrap().expect("batch");
    assert_eq!(first.frame_count, 2);
    assert_eq!(first.poll_count, 1);
    assert_eq!(first.total_frames, 2);

    // Poll interval gate blocks immediate second poll.
    assert!(runtime.poll_batch(8, 1_100).unwrap().is_none());

    let second = runtime.poll_batch(8, 2_200).unwrap().expect("batch");
    assert_eq!(second.poll_count, 2);
    assert_eq!(second.total_frames, 4);

    runtime.stop_if_running().unwrap();
    assert!(!runtime.running());
}
