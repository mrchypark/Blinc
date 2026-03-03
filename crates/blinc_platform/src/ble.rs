use std::collections::BTreeMap;

use blinc_core::native_bridge::native_call;
use serde::{Deserialize, Serialize};

use crate::error::PlatformError;

/// BLE scan configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BleScanConfig {
    /// Optional service UUID filters.
    pub service_uuids: Vec<String>,
    /// Allow duplicate scan results.
    pub allow_duplicates: bool,
    /// Optional low-level scan mode hint.
    pub scan_mode: Option<String>,
    /// Native flush interval in milliseconds.
    pub frame_flush_ms: u16,
}

impl Default for BleScanConfig {
    fn default() -> Self {
        Self {
            service_uuids: Vec::new(),
            allow_duplicates: false,
            scan_mode: None,
            frame_flush_ms: 500,
        }
    }
}

/// A normalized BLE scan result frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BleScanResult {
    pub seq: u64,
    pub address: String,
    pub name: Option<String>,
    pub rssi: i32,
    pub tx_power: Option<i32>,
    pub is_connectable: Option<bool>,
    pub service_uuids: Vec<String>,
    pub manufacturer_data: Option<String>,
    pub service_data: Option<String>,
    pub time_monotonic_ns: u64,
    pub time_unix_ms: i64,
}

/// Snapshot of current BLE scan backend state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BleScanStatus {
    pub running: bool,
    pub buffered_results: usize,
    pub active_session_id: Option<String>,
}

impl Default for BleScanStatus {
    fn default() -> Self {
        Self {
            running: false,
            buffered_results: 0,
            active_session_id: None,
        }
    }
}

/// Pluggable backend abstraction for BLE scan operations.
pub trait BleBackend: Send + Sync {
    fn configure(&self, config: &BleScanConfig) -> Result<(), PlatformError>;
    fn start_scan(&self, session_id: &str) -> Result<(), PlatformError>;
    fn stop_scan(&self, session_id: &str) -> Result<(), PlatformError>;
    fn status(&self) -> Result<BleScanStatus, PlatformError>;
    fn drain_results(&self, max_results: usize) -> Result<Vec<BleScanResult>, PlatformError>;
}

/// Native bridge BLE backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeBridgeBleBackend;

impl BleBackend for NativeBridgeBleBackend {
    fn configure(&self, config: &BleScanConfig) -> Result<(), PlatformError> {
        let json = serde_json::to_string(config)?;
        let ok: bool = native_call("ble", "configure", (json,))?;
        if ok {
            Ok(())
        } else {
            Err(PlatformError::Other(
                "native ble.configure returned false".to_string(),
            ))
        }
    }

    fn start_scan(&self, session_id: &str) -> Result<(), PlatformError> {
        let ok: bool = native_call("ble", "start", (session_id.to_string(),))?;
        if ok {
            Ok(())
        } else {
            Err(PlatformError::Other(
                "native ble.start returned false".to_string(),
            ))
        }
    }

    fn stop_scan(&self, session_id: &str) -> Result<(), PlatformError> {
        let ok: bool = native_call("ble", "stop", (session_id.to_string(),))?;
        if ok {
            Ok(())
        } else {
            Err(PlatformError::Other(
                "native ble.stop returned false".to_string(),
            ))
        }
    }

    fn status(&self) -> Result<BleScanStatus, PlatformError> {
        let json: String = native_call("ble", "status", ())?;
        Ok(serde_json::from_str(&json)?)
    }

    fn drain_results(&self, max_results: usize) -> Result<Vec<BleScanResult>, PlatformError> {
        let max_results_i32 = i32::try_from(max_results)
            .map_err(|_| PlatformError::Other("max_results exceeds i32 range".to_string()))?;
        let json: String = native_call("ble", "drain_results", (max_results_i32,))?;
        Ok(serde_json::from_str(&json)?)
    }
}

/// High-level typed client for BLE scan operations.
pub struct BleClient<B: BleBackend> {
    backend: B,
}

impl<B: BleBackend> BleClient<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn configure(&self, config: &BleScanConfig) -> Result<(), PlatformError> {
        if config.frame_flush_ms == 0 {
            return Err(PlatformError::Other(
                "frame_flush_ms must be greater than 0".to_string(),
            ));
        }
        self.backend.configure(config)
    }

    pub fn start_scan(&self, session_id: &str) -> Result<(), PlatformError> {
        if session_id.trim().is_empty() {
            return Err(PlatformError::Other(
                "session id must not be empty".to_string(),
            ));
        }
        self.backend.start_scan(session_id)
    }

    pub fn stop_scan(&self, session_id: &str) -> Result<(), PlatformError> {
        if session_id.trim().is_empty() {
            return Err(PlatformError::Other(
                "session id must not be empty".to_string(),
            ));
        }
        self.backend.stop_scan(session_id)
    }

    pub fn status(&self) -> Result<BleScanStatus, PlatformError> {
        self.backend.status()
    }

    pub fn drain_results(&self, max_results: usize) -> Result<Vec<BleScanResult>, PlatformError> {
        self.backend.drain_results(max_results)
    }
}

/// Per-batch summary for BLE polling.
#[derive(Debug, Clone)]
pub struct BleBatchSummary {
    pub poll_count: u64,
    pub total_results: u64,
    pub result_count: usize,
    pub counts_by_address: BTreeMap<String, usize>,
    pub sample: Option<BleScanResult>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BleProbeState {
    pub last_poll_ms: u64,
    pub total_results: u64,
    pub poll_count: u64,
}

/// Runtime controller for BLE scan session lifecycle + polling.
pub struct BleRuntimeController<B: BleBackend> {
    client: BleClient<B>,
    session_id: String,
    running: bool,
    poll_interval_ms: u64,
    probe: BleProbeState,
}

impl<B: BleBackend> BleRuntimeController<B> {
    pub fn new(client: BleClient<B>, session_id: impl Into<String>) -> Self {
        Self {
            client,
            session_id: session_id.into(),
            running: false,
            poll_interval_ms: 1_000,
            probe: BleProbeState::default(),
        }
    }

    pub fn client(&self) -> &BleClient<B> {
        &self.client
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn set_poll_interval_ms(&mut self, interval_ms: u64) {
        self.poll_interval_ms = interval_ms.max(1);
    }

    pub fn ensure_started(&mut self) -> Result<(), PlatformError> {
        if self.running {
            return Ok(());
        }
        self.client.start_scan(&self.session_id)?;
        self.running = true;
        Ok(())
    }

    pub fn stop_if_running(&mut self) -> Result<(), PlatformError> {
        if !self.running {
            return Ok(());
        }
        self.client.stop_scan(&self.session_id)?;
        self.running = false;
        Ok(())
    }

    pub fn poll_batch(
        &mut self,
        max_results: usize,
        now_ms: u64,
    ) -> Result<Option<BleBatchSummary>, PlatformError> {
        if !self.running {
            return Ok(None);
        }

        if self.probe.last_poll_ms > 0
            && now_ms.saturating_sub(self.probe.last_poll_ms) < self.poll_interval_ms
        {
            return Ok(None);
        }
        self.probe.last_poll_ms = now_ms;

        let results = self.client.drain_results(max_results)?;
        if results.is_empty() {
            return Ok(None);
        }

        self.probe.poll_count += 1;
        self.probe.total_results += results.len() as u64;

        let mut counts_by_address: BTreeMap<String, usize> = BTreeMap::new();
        for result in &results {
            *counts_by_address.entry(result.address.clone()).or_insert(0) += 1;
        }

        Ok(Some(BleBatchSummary {
            poll_count: self.probe.poll_count,
            total_results: self.probe.total_results,
            result_count: results.len(),
            counts_by_address,
            sample: results.last().cloned(),
        }))
    }
}

/// Native bridge backend path for BLE module.
pub mod native_bridge {
    pub use super::NativeBridgeBleBackend;
}
