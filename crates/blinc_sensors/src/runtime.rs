use std::collections::BTreeMap;

use crate::{
    SensorBackend, SensorClient, SensorConfig, SensorError, SensorFrame, SensorKind,
    SensorPermissionBackend, SensorPermissionService,
};

/// Per-batch summary for sensor polling.
#[derive(Debug, Clone)]
pub struct SensorBatchSummary {
    pub poll_count: u64,
    pub total_frames: u64,
    pub frame_count: usize,
    pub counts: BTreeMap<SensorKind, usize>,
    pub sample: Option<SensorFrame>,
}

impl SensorBatchSummary {
    pub fn counts_compact(&self) -> String {
        let mut parts = Vec::with_capacity(self.counts.len());
        for (kind, count) in &self.counts {
            parts.push(format!("{}={}", kind.as_str(), count));
        }
        parts.join(", ")
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SensorProbeState {
    pub last_poll_ms: u64,
    pub total_frames: u64,
    pub poll_count: u64,
}

/// Cross-platform runtime controller for session lifecycle + polling.
pub struct SensorRuntimeController<B: SensorBackend, P: SensorPermissionBackend> {
    client: SensorClient<B>,
    permissions: SensorPermissionService<P>,
    session_id: String,
    running: bool,
    poll_interval_ms: u64,
    probe: SensorProbeState,
}

impl<B: SensorBackend, P: SensorPermissionBackend> SensorRuntimeController<B, P> {
    pub fn new(
        client: SensorClient<B>,
        permissions: SensorPermissionService<P>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            permissions,
            session_id: session_id.into(),
            running: false,
            poll_interval_ms: 1_000,
            probe: SensorProbeState::default(),
        }
    }

    pub fn client(&self) -> &SensorClient<B> {
        &self.client
    }

    pub fn configure(&self, config: &SensorConfig) -> Result<(), SensorError> {
        self.client.configure(config)
    }

    pub fn supported_kinds(&self) -> Result<Vec<SensorKind>, SensorError> {
        self.client.supported_kinds()
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn set_poll_interval_ms(&mut self, interval_ms: u64) {
        self.poll_interval_ms = interval_ms.max(1);
    }

    pub fn ensure_started(&mut self) -> Result<(), SensorError> {
        if self.running {
            return Ok(());
        }

        // Start only after required permissions are confirmed.
        let permission_state = self.permissions.request_required_permissions()?;
        if !permission_state.ready() {
            return Err(SensorError::required_permissions_not_ready(
                permission_state,
            ));
        }

        self.client.start_session(&self.session_id)?;
        self.running = true;
        Ok(())
    }

    pub fn stop_if_running(&mut self) -> Result<(), SensorError> {
        if !self.running {
            return Ok(());
        }
        self.client.stop_session(&self.session_id)?;
        self.running = false;
        Ok(())
    }

    pub fn poll_batch(
        &mut self,
        max_frames: usize,
        now_ms: u64,
    ) -> Result<Option<SensorBatchSummary>, SensorError> {
        if !self.running {
            return Ok(None);
        }

        if self.probe.last_poll_ms > 0
            && now_ms.saturating_sub(self.probe.last_poll_ms) < self.poll_interval_ms
        {
            return Ok(None);
        }
        self.probe.last_poll_ms = now_ms;

        let frames = self.client.drain_frames(max_frames)?;
        if frames.is_empty() {
            return Ok(None);
        }

        self.probe.poll_count += 1;
        self.probe.total_frames += frames.len() as u64;

        let mut counts: BTreeMap<SensorKind, usize> = BTreeMap::new();
        for frame in &frames {
            *counts.entry(frame.sensor).or_insert(0) += 1;
        }

        Ok(Some(SensorBatchSummary {
            poll_count: self.probe.poll_count,
            total_frames: self.probe.total_frames,
            frame_count: frames.len(),
            counts,
            sample: frames.last().cloned(),
        }))
    }
}
