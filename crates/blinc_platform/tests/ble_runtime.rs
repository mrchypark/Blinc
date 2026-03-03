use std::sync::Mutex;

use blinc_platform::ble::{
    BleBackend, BleClient, BleRuntimeController, BleScanConfig, BleScanResult, BleScanStatus,
};
use blinc_platform::PlatformError;

#[derive(Default)]
struct MockBleBackend {
    running: Mutex<bool>,
    drained: Mutex<u64>,
}

impl BleBackend for MockBleBackend {
    fn configure(&self, _config: &BleScanConfig) -> Result<(), PlatformError> {
        Ok(())
    }

    fn start_scan(&self, _session_id: &str) -> Result<(), PlatformError> {
        *self.running.lock().expect("running lock") = true;
        Ok(())
    }

    fn stop_scan(&self, _session_id: &str) -> Result<(), PlatformError> {
        *self.running.lock().expect("running lock") = false;
        Ok(())
    }

    fn status(&self) -> Result<BleScanStatus, PlatformError> {
        Ok(BleScanStatus {
            running: *self.running.lock().expect("running lock"),
            buffered_results: 0,
            active_session_id: Some("ble-test".to_string()),
        })
    }

    fn drain_results(&self, max_results: usize) -> Result<Vec<BleScanResult>, PlatformError> {
        if !*self.running.lock().expect("running lock") {
            return Ok(vec![]);
        }
        let mut drained = self.drained.lock().expect("drained lock");
        let mut out = Vec::new();
        for idx in 0..max_results.min(2) {
            *drained += 1;
            out.push(BleScanResult {
                seq: *drained,
                address: format!("AA:BB:CC:00:00:{:02X}", idx),
                name: Some(format!("dev-{}", idx)),
                rssi: -40 - idx as i32,
                tx_power: None,
                is_connectable: Some(true),
                service_uuids: vec![],
                manufacturer_data: None,
                service_data: None,
                time_monotonic_ns: 1_000 + *drained,
                time_unix_ms: 1_700_000_000_000 + *drained as i64,
            });
        }
        Ok(out)
    }
}

#[test]
fn ble_runtime_starts_polls_and_stops() {
    let client = BleClient::new(MockBleBackend::default());
    client
        .configure(&BleScanConfig::default())
        .expect("configure");

    let mut runtime = BleRuntimeController::new(client, "ble-runtime");
    runtime.set_poll_interval_ms(500);

    runtime.ensure_started().expect("start");
    assert!(runtime.running());

    let first = runtime.poll_batch(8, 1_000).expect("poll").expect("batch");
    assert_eq!(first.result_count, 2);
    assert_eq!(first.poll_count, 1);
    assert_eq!(first.total_results, 2);

    assert!(runtime.poll_batch(8, 1_200).expect("gated poll").is_none());

    let second = runtime
        .poll_batch(8, 2_000)
        .expect("poll 2")
        .expect("batch 2");
    assert_eq!(second.poll_count, 2);
    assert_eq!(second.total_results, 4);

    runtime.stop_if_running().expect("stop");
    assert!(!runtime.running());
}
