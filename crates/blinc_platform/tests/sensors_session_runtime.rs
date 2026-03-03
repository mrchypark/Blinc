use std::sync::Mutex;

use blinc_platform::sensors::{
    SensorAccuracy, SensorBackend, SensorClient, SensorConfig, SensorError, SensorFrame,
    SensorKind, SensorStatus,
};

#[derive(Default)]
struct QueueBackend {
    running: Mutex<bool>,
    next_seq: Mutex<u64>,
}

impl SensorBackend for QueueBackend {
    fn configure(&self, _config: &SensorConfig) -> Result<(), SensorError> {
        Ok(())
    }

    fn start(&self, _session_id: &str) -> Result<(), SensorError> {
        *self.running.lock().expect("running lock") = true;
        Ok(())
    }

    fn stop(&self, _session_id: &str) -> Result<(), SensorError> {
        *self.running.lock().expect("running lock") = false;
        Ok(())
    }

    fn status(&self) -> Result<SensorStatus, SensorError> {
        Ok(SensorStatus {
            running: *self.running.lock().expect("running lock"),
            buffered_frames: 0,
            active_session_id: None,
        })
    }

    fn drain_frames(&self, max_frames: usize) -> Result<Vec<SensorFrame>, SensorError> {
        if !*self.running.lock().expect("running lock") {
            return Ok(Vec::new());
        }

        let count = max_frames.min(3);
        let mut seq = self.next_seq.lock().expect("seq lock");
        let mut out = Vec::with_capacity(count);
        for index in 0..count {
            *seq += 1;
            out.push(SensorFrame {
                seq: *seq,
                sensor: if index % 2 == 0 {
                    SensorKind::Accelerometer
                } else {
                    SensorKind::Gyroscope
                },
                time_monotonic_ns: 1_000_000_000 + *seq,
                time_unix_ms: 1_700_000_000_000 + *seq as i64,
                accuracy: SensorAccuracy::High,
                values: vec![0.1 + index as f32, 0.2, 0.3],
            });
        }
        Ok(out)
    }

    fn supported_kinds(&self) -> Result<Vec<SensorKind>, SensorError> {
        Ok(vec![
            SensorKind::Gps,
            SensorKind::Accelerometer,
            SensorKind::Gyroscope,
            SensorKind::Activity,
        ])
    }
}

#[test]
fn session_collects_frames_and_stops_cleanly() {
    let client = SensorClient::new(QueueBackend::default());
    client
        .configure(&SensorConfig::default())
        .expect("configure");

    assert!(!client.status().expect("status").running);
    assert_eq!(client.supported_kinds().expect("supported kinds").len(), 4);

    client.start_session("runtime-check").expect("start");
    assert!(client.status().expect("status").running);

    let frames = client.drain_frames(2).expect("drain");
    assert_eq!(frames.len(), 2);
    assert!(frames[0].seq < frames[1].seq);
    assert!(!frames[0].values.is_empty());

    client.stop_session("runtime-check").expect("stop");
    assert!(!client.status().expect("status").running);
    assert!(client.drain_frames(8).expect("drain").is_empty());
}
