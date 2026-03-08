//! Main application module for the debugger.

use crate::panels::{
    CommandPanel, EvidencePanel, InspectorPanel, PreviewConfig, PreviewPanel, TimelinePanel,
    TimelinePanelState, TreePanel, TreePanelState,
};
use crate::theme::DebuggerColors;
use anyhow::{anyhow, bail, Context, Result};
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_app::WindowConfig;
use blinc_layout::prelude::*;
use blinc_recorder::replay::{
    FrameUpdate, ReplayConfig, ReplayPlayer, ReplayState, SimulatedInput,
};
use blinc_recorder::{
    RecordedEvent, RecordingExport, Timestamp, TimestampedEvent, TraceEntry, TraceEntryKind,
    TreeSnapshot,
};
use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

const MAX_NETWORK_PAYLOAD_BYTES: usize = 100 * 1024 * 1024;
const MAX_EXPORT_STREAM_PAYLOAD_BYTES: usize = MAX_NETWORK_PAYLOAD_BYTES;
const MAX_SERVER_MESSAGES_TO_PARSE: usize = 32;

/// Application state
#[derive(Default)]
pub struct AppState {
    /// Loaded recording (if any)
    pub recording: Option<RecordingExport>,
    /// Replay player (if recording loaded)
    pub player: Option<Arc<Mutex<ReplayPlayer>>>,
    /// Current tree snapshot
    pub current_snapshot: Option<TreeSnapshot>,
    /// Selected element ID
    pub selected_element_id: Option<String>,
    /// Tree panel state
    pub tree_state: TreePanelState,
    /// Preview config
    pub preview_config: PreviewConfig,
    /// Timeline state
    pub timeline_state: TimelinePanelState,
    /// Last known cursor position from replay
    pub cursor_position: Option<(f32, f32)>,
    /// Server address
    pub server_addr: Option<String>,
    /// Cached command stream lines for the current recording.
    pub command_lines: Arc<[String]>,
    /// Cached evidence lines for the current recording.
    pub evidence_lines: Arc<[String]>,
    /// Cached locator/assertion context for the selected element.
    pub selected_trace_context: SelectedTraceContext,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SelectedTraceContext {
    pub locator_lines: Arc<[String]>,
    pub assertion_lines: Arc<[String]>,
}

impl AppState {
    /// Load a recording from file.
    pub fn load_recording(&mut self, path: &PathBuf) -> Result<()> {
        let contents = std::fs::read_to_string(path)?;
        let export: RecordingExport = serde_json::from_str(&contents)?;
        self.apply_recording(export);
        log::info!("Loaded recording from {}", path.display());
        Ok(())
    }

    /// Load recording data received from server.
    pub fn load_from_server(&mut self, addr: &str, export: RecordingExport) {
        self.apply_recording(export);
        self.server_addr = Some(addr.to_string());
        log::info!("Loaded recording from server: {addr}");
    }

    /// Get the selected element snapshot.
    pub fn selected_element(&self) -> Option<&blinc_recorder::ElementSnapshot> {
        let snapshot = self.current_snapshot.as_ref()?;
        let id = self.selected_element_id.as_ref()?;
        snapshot.elements.get(id)
    }

    fn build_command_lines(recording: &RecordingExport, cutoff: Timestamp) -> Arc<[String]> {
        recording
            .trace_entries
            .iter()
            .filter(|entry| entry.timestamp <= cutoff)
            .filter_map(|entry| match &entry.kind {
                TraceEntryKind::Command(command) => {
                    let mut line = match command.target.as_deref() {
                        Some(target) => format!("{} -> {}", command.name, target),
                        None => command.name.clone(),
                    };
                    if let Some(payload) = command.payload.as_deref() {
                        line.push_str(&format!(" [{payload}]"));
                    }
                    Some(line)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn build_evidence_lines(recording: &RecordingExport, cutoff: Timestamp) -> Arc<[String]> {
        recording
            .trace_entries
            .iter()
            .filter(|entry| entry.timestamp <= cutoff)
            .filter_map(|entry| match &entry.kind {
                TraceEntryKind::LocatorResolution(resolution) => Some(format!(
                    "locator {}{} [{}]{}",
                    resolution.query,
                    resolution
                        .matched_target
                        .as_deref()
                        .map(|target| format!(" -> {target}"))
                        .unwrap_or_default(),
                    resolution.failure_reason.as_deref().unwrap_or_else(|| {
                        if resolution.matched_target.is_some() {
                            "matched"
                        } else {
                            "unresolved"
                        }
                    }),
                    if resolution.candidate_targets.is_empty() {
                        String::new()
                    } else {
                        format!(" candidates={}", resolution.candidate_targets.join(", "))
                    }
                )),
                TraceEntryKind::Assertion(assertion) => Some(format!(
                    "{} {}{}",
                    if assertion.passed { "PASS" } else { "FAIL" },
                    assertion.code,
                    assertion
                        .target
                        .as_deref()
                        .map(|target| format!(" -> {target}"))
                        .unwrap_or_default()
                )),
                TraceEntryKind::Artifact(artifact) => Some(format!(
                    "artifact {}{}{}",
                    artifact.kind,
                    artifact
                        .message
                        .as_deref()
                        .map(|message| format!(": {message}"))
                        .unwrap_or_default(),
                    artifact
                        .path
                        .as_deref()
                        .map(|path| format!(" @ {path}"))
                        .unwrap_or_default()
                )),
                _ => None,
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn build_selected_trace_context(
        recording: Option<&RecordingExport>,
        snapshot: Option<&TreeSnapshot>,
        selected_id: Option<&str>,
        cutoff: Timestamp,
    ) -> SelectedTraceContext {
        let (Some(selected_id), Some(recording)) = (selected_id, recording) else {
            return SelectedTraceContext::default();
        };
        let aliases = selected_trace_aliases(snapshot, selected_id);

        let locator_lines = recording
            .trace_entries
            .iter()
            .filter(|entry| entry.timestamp <= cutoff)
            .filter_map(|entry| match &entry.kind {
                TraceEntryKind::LocatorResolution(resolution)
                    if resolution
                        .matched_target
                        .as_deref()
                        .is_some_and(|target| aliases.contains(target))
                        || resolution
                            .candidate_targets
                            .iter()
                            .any(|target| aliases.contains(target)) =>
                {
                    let status = if resolution
                        .matched_target
                        .as_deref()
                        .is_some_and(|target| aliases.contains(target))
                    {
                        resolution.failure_reason.as_deref().unwrap_or("matched")
                    } else {
                        "candidate"
                    };
                    Some(format!("{} [{}]", resolution.query, status))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .into();
        let assertion_lines = recording
            .trace_entries
            .iter()
            .filter(|entry| entry.timestamp <= cutoff)
            .filter_map(|entry| match &entry.kind {
                TraceEntryKind::Assertion(assertion)
                    if assertion
                        .target
                        .as_deref()
                        .is_some_and(|target| aliases.contains(target)) =>
                {
                    Some(format!(
                        "{} {}{}{}",
                        if assertion.passed { "PASS" } else { "FAIL" },
                        assertion.code,
                        assertion
                            .expected
                            .as_deref()
                            .map(|expected| format!(" expected={expected:?}"))
                            .unwrap_or_default(),
                        assertion
                            .actual
                            .as_deref()
                            .map(|actual| format!(" actual={actual:?}"))
                            .unwrap_or_default()
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .into();

        SelectedTraceContext {
            locator_lines,
            assertion_lines,
        }
    }

    fn refresh_trace_lists(&mut self) {
        if let Some(recording) = self.recording.as_ref() {
            let cutoff = self.trace_cutoff();
            self.command_lines = Self::build_command_lines(recording, cutoff);
            self.evidence_lines = Self::build_evidence_lines(recording, cutoff);
        } else {
            self.command_lines = Arc::default();
            self.evidence_lines = Arc::default();
        }
    }

    fn refresh_selected_trace_context(&mut self) {
        self.selected_trace_context = Self::build_selected_trace_context(
            self.recording.as_ref(),
            self.current_snapshot.as_ref(),
            self.selected_element_id.as_deref(),
            self.trace_cutoff(),
        );
    }

    fn refresh_trace_views(&mut self) {
        self.refresh_trace_lists();
        self.refresh_selected_trace_context();
    }

    fn trace_cutoff(&self) -> Timestamp {
        self.timeline_state.position
    }

    fn sync_snapshot_to_position(&mut self, position: Timestamp, snapshots: &[TreeSnapshot]) {
        self.current_snapshot = snapshots
            .iter()
            .rfind(|snapshot| snapshot.timestamp <= position)
            .cloned();
    }

    fn sync_cursor_to_position(&mut self, position: Timestamp, events: &[TimestampedEvent]) {
        self.cursor_position = events
            .iter()
            .filter(|event| event.timestamp <= position)
            .fold(None, |cursor, event| match &event.event {
                RecordedEvent::MouseDown(event) => Some((event.position.x, event.position.y)),
                RecordedEvent::MouseUp(event) => Some((event.position.x, event.position.y)),
                RecordedEvent::Click(event) => Some((event.position.x, event.position.y)),
                RecordedEvent::DoubleClick(event) => Some((event.position.x, event.position.y)),
                RecordedEvent::MouseMove(event) => Some((event.position.x, event.position.y)),
                RecordedEvent::Scroll(event) => Some((event.position.x, event.position.y)),
                RecordedEvent::HoverEnter(event) => Some((event.position.x, event.position.y)),
                RecordedEvent::HoverLeave(event) => Some((event.position.x, event.position.y)),
                _ => cursor,
            });
    }

    pub fn set_selected_element_id(&mut self, id: Option<String>) {
        self.selected_element_id = id;
        self.refresh_selected_trace_context();
    }

    pub fn command_stream_lines(&self) -> Arc<[String]> {
        Arc::clone(&self.command_lines)
    }

    pub fn evidence_lines(&self) -> Arc<[String]> {
        Arc::clone(&self.evidence_lines)
    }

    pub fn selected_trace_context(&self) -> SelectedTraceContext {
        self.selected_trace_context.clone()
    }

    /// Tick replay state once per UI build.
    pub fn tick(&mut self) {
        let player_arc = self.player.clone();
        if let Some(player_arc) = player_arc {
            let mut player = match player_arc.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };

            if player.state() == ReplayState::Playing {
                let update = player.update();
                self.apply_frame_update(update);
                let snapshots = player.all_snapshots().to_vec();
                let position = player.position();
                let events = self
                    .recording
                    .as_ref()
                    .map(|recording| recording.events.clone());
                self.sync_snapshot_to_position(position, &snapshots);
                if let Some(events) = events {
                    self.sync_cursor_to_position(position, &events);
                }
            } else {
                let snapshots = player.all_snapshots().to_vec();
                let position = player.position();
                let events = self
                    .recording
                    .as_ref()
                    .map(|recording| recording.events.clone());
                self.sync_snapshot_to_position(position, &snapshots);
                if let Some(events) = events {
                    self.sync_cursor_to_position(position, &events);
                }
            }

            self.timeline_state.position = player.position();
            self.timeline_state.duration = player.duration();
            self.timeline_state.playback_state = player.state();
            self.timeline_state.speed = player.clock().speed();
        }

        self.ensure_selected_element_exists();
        self.refresh_trace_views();
    }

    pub fn toggle_playback(&mut self) {
        self.with_player(|state, player| {
            player.toggle();
            state.timeline_state.playback_state = player.state();
            state.refresh_trace_views();
        });
    }

    pub fn step_back(&mut self) {
        self.with_player(|state, player| {
            let update = player.step_back();
            state.apply_frame_update(update);
            let position = player.position();
            state.timeline_state.position = position;
            state.timeline_state.playback_state = player.state();
            let snapshots = player.all_snapshots().to_vec();
            let events = state
                .recording
                .as_ref()
                .map(|recording| recording.events.clone());
            state.sync_snapshot_to_position(position, &snapshots);
            if let Some(events) = events {
                state.sync_cursor_to_position(position, &events);
            }
            state.refresh_trace_views();
        });
    }

    pub fn step_forward(&mut self) {
        self.with_player(|state, player| {
            let update = player.step();
            state.apply_frame_update(update);
            let position = player.position();
            state.timeline_state.position = position;
            state.timeline_state.playback_state = player.state();
            let snapshots = player.all_snapshots().to_vec();
            let events = state
                .recording
                .as_ref()
                .map(|recording| recording.events.clone());
            state.sync_snapshot_to_position(position, &snapshots);
            if let Some(events) = events {
                state.sync_cursor_to_position(position, &events);
            }
            state.refresh_trace_views();
        });
    }

    pub fn seek_normalized(&mut self, normalized: f32) {
        self.with_player(|state, player| {
            let micros =
                (player.duration().as_micros() as f32 * normalized.clamp(0.0, 1.0)).round() as u64;
            player.seek(Timestamp::from_micros(micros));
            let position = player.position();
            state.timeline_state.position = position;
            state.timeline_state.playback_state = ReplayState::Paused;
            let snapshots = player.all_snapshots().to_vec();
            let events = state
                .recording
                .as_ref()
                .map(|recording| recording.events.clone());
            state.sync_snapshot_to_position(position, &snapshots);
            if let Some(events) = events {
                state.sync_cursor_to_position(position, &events);
            }
            state.refresh_trace_views();
        });
    }

    pub fn set_playback_speed(&mut self, speed: f64) {
        self.with_player(|state, player| {
            player.clock_mut().set_speed(speed);
            state.timeline_state.speed = player.clock().speed();
        });
    }

    fn apply_recording(&mut self, export: RecordingExport) {
        let player = ReplayPlayer::new(export.clone(), ReplayConfig::interactive());
        self.timeline_state.duration = player.duration();
        self.timeline_state.position = Timestamp::zero();
        self.timeline_state.playback_state = ReplayState::Idle;
        self.timeline_state.speed = player.clock().speed();

        self.sync_snapshot_to_position(self.timeline_state.position, &export.snapshots);
        self.selected_element_id = self.current_snapshot.as_ref().and_then(|s| {
            s.root_id
                .clone()
                .or_else(|| s.elements.keys().next().cloned())
        });
        self.sync_cursor_to_position(self.timeline_state.position, &export.events);

        self.recording = Some(export);
        self.refresh_trace_views();
        self.player = Some(Arc::new(Mutex::new(player)));
    }

    fn apply_frame_update(&mut self, update: FrameUpdate) {
        if let Some(snapshot) = update.snapshot {
            self.current_snapshot = Some(snapshot);
        }

        for event in update.events {
            match event {
                SimulatedInput::Click { position, .. }
                | SimulatedInput::DoubleClick { position, .. }
                | SimulatedInput::MouseDown { position, .. }
                | SimulatedInput::MouseUp { position, .. }
                | SimulatedInput::MouseMove { position, .. }
                | SimulatedInput::Scroll { position, .. }
                | SimulatedInput::HoverEnter { position, .. }
                | SimulatedInput::HoverLeave { position, .. } => {
                    self.cursor_position = Some((position.x, position.y));
                }
                _ => {}
            }
        }
    }

    fn ensure_selected_element_exists(&mut self) {
        let Some(snapshot) = self.current_snapshot.as_ref() else {
            self.set_selected_element_id(None);
            return;
        };

        let selected_exists = self
            .selected_element_id
            .as_ref()
            .is_some_and(|id| snapshot.elements.contains_key(id));
        if !selected_exists {
            let next = snapshot
                .root_id
                .clone()
                .or_else(|| snapshot.elements.keys().next().cloned());
            self.set_selected_element_id(next);
        }
    }

    fn with_player<F>(&mut self, action: F)
    where
        F: FnOnce(&mut Self, &mut ReplayPlayer),
    {
        let Some(player_arc) = self.player.clone() else {
            return;
        };
        let Ok(mut player) = player_arc.lock() else {
            return;
        };
        action(self, &mut player);
    }
}

/// Shared application state for thread-safe access.
pub type SharedAppState = Arc<RwLock<AppState>>;

/// Run the debugger application.
pub fn run(width: u32, height: u32, file: Option<PathBuf>, connect: Option<String>) -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::default()));

    if let Some(ref path) = file {
        match app_state.write() {
            Ok(mut state) => {
                if let Err(e) = state.load_recording(path) {
                    log::warn!("Failed to load recording from {:?}: {}", path, e);
                }
            }
            Err(e) => {
                log::error!("App state lock is poisoned, cannot load recording: {e}");
            }
        }
    }

    if let Some(ref addr) = connect {
        match request_export_from_server(addr) {
            Ok(export) => match app_state.write() {
                Ok(mut state) => state.load_from_server(addr, export),
                Err(e) => {
                    log::error!("App state lock is poisoned, cannot load from server: {e}");
                }
            },
            Err(e) => log::warn!("Failed to load recording from server {}: {}", addr, e),
        }
    }

    let config = WindowConfig {
        title: "Blinc Debugger".to_string(),
        width,
        height,
        resizable: true,
        ..Default::default()
    };

    let state_for_ui = app_state.clone();
    Ok(WindowedApp::run(config, move |ctx| {
        build_debugger_ui(ctx, &state_for_ui)
    })?)
}

/// Build the debugger UI.
fn build_debugger_ui(ctx: &WindowedContext, app_state: &SharedAppState) -> impl ElementBuilder {
    if let Ok(mut state) = app_state.write() {
        state.tick();
    } else {
        log::error!("App state lock is poisoned during tick");
        return unavailable_debugger_ui(ctx);
    }
    let state = match app_state.read() {
        Ok(state) => state,
        Err(e) => {
            log::error!("App state lock is poisoned during render: {e}");
            return unavailable_debugger_ui(ctx);
        }
    };

    let on_tree_select = make_state_callback(app_state, |state, id: String| {
        state.set_selected_element_id(Some(id));
    });
    let on_toggle_bounds = make_state_callback(app_state, |state, value: bool| {
        state.preview_config.show_bounds = value;
    });
    let on_toggle_cursor = make_state_callback(app_state, |state, value: bool| {
        state.preview_config.show_cursor = value;
    });
    let on_zoom = make_state_callback(app_state, |state, value: f32| {
        state.preview_config.zoom = value;
    });
    let on_step_back = make_state_callback0(app_state, |state| {
        state.step_back();
    });
    let on_play_pause = make_state_callback0(app_state, |state| {
        state.toggle_playback();
    });
    let on_step_forward = make_state_callback0(app_state, |state| {
        state.step_forward();
    });
    let on_seek = make_state_callback(app_state, |state, normalized: f32| {
        state.seek_normalized(normalized);
    });
    let on_speed_change = make_state_callback(app_state, |state, speed: f64| {
        state.set_playback_speed(speed);
    });
    let command_lines = state.command_stream_lines();
    let evidence_lines = state.evidence_lines();
    let trace_context = state.selected_trace_context();
    let command_scroll = ctx.use_scroll_ref("debugger.command_stream");
    let inspector_scroll = ctx.use_scroll_ref("debugger.inspector");
    let evidence_scroll = ctx.use_scroll_ref("debugger.evidence");

    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(DebuggerColors::bg_base())
        .flex_col()
        .child(
            div()
                .flex_grow()
                .flex_row()
                .child(TreePanel::new(
                    state.current_snapshot.as_ref(),
                    state.selected_element_id.as_ref(),
                    &state.tree_state,
                    Some(on_tree_select),
                ))
                .child(
                    div()
                        .flex_grow()
                        .flex_col()
                        .child(PreviewPanel::new(
                            state.current_snapshot.as_ref(),
                            &state.preview_config,
                            state.cursor_position,
                            Some(on_toggle_bounds),
                            Some(on_toggle_cursor),
                            Some(on_zoom),
                        ))
                        .child(CommandPanel::with_scroll_ref(command_lines, command_scroll)),
                )
                .child(
                    div()
                        .h_full()
                        .flex_col()
                        .child(InspectorPanel::new(
                            state.selected_element(),
                            trace_context,
                            inspector_scroll,
                        ))
                        .child(EvidencePanel::new(evidence_lines, evidence_scroll)),
                ),
        )
        .child(TimelinePanel::new(
            state
                .recording
                .as_ref()
                .map(|r| r.events.as_slice())
                .unwrap_or(&[]),
            state
                .recording
                .as_ref()
                .map(|r| r.trace_entries.as_slice())
                .unwrap_or(&[]),
            &state.timeline_state,
            Some(on_step_back),
            Some(on_play_pause),
            Some(on_step_forward),
            Some(on_seek),
            Some(on_speed_change),
        ))
}

fn unavailable_debugger_ui(ctx: &WindowedContext) -> Div {
    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(DebuggerColors::bg_base())
        .items_center()
        .justify_center()
        .child(text("Debugger state unavailable"))
}

fn read_len_prefixed<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_NETWORK_PAYLOAD_BYTES {
        bail!(
            "payload size {} exceeds maximum allowed {}",
            len,
            MAX_NETWORK_PAYLOAD_BYTES
        );
    }
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

fn write_len_prefixed<W: Write>(writer: &mut W, payload: &[u8]) -> Result<()> {
    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    Ok(())
}

fn request_export_from_server(addr: &str) -> Result<RecordingExport> {
    #[cfg(unix)]
    {
        match resolve_connect_target(addr)
            .with_context(|| format!("invalid --connect target: {addr}"))?
        {
            ConnectTarget::Unix(socket) => {
                return request_export_over_unix_socket(&socket)
                    .with_context(|| format!("failed to connect to unix socket {socket}"));
            }
            ConnectTarget::Tcp(target) => {
                return request_export_over_tcp(&target)
                    .with_context(|| format!("failed to connect to tcp server {target}"));
            }
        }
    }

    request_export_over_tcp(addr).with_context(|| format!("failed to connect to tcp server {addr}"))
}

fn make_state_callback<T, F>(app_state: &SharedAppState, action: F) -> Arc<dyn Fn(T) + Send + Sync>
where
    T: Send + 'static,
    F: Fn(&mut AppState, T) + Send + Sync + 'static,
{
    let shared = app_state.clone();
    Arc::new(move |value: T| {
        if let Ok(mut state) = shared.write() {
            action(&mut state, value);
        }
    })
}

fn selected_trace_aliases(snapshot: Option<&TreeSnapshot>, selected_id: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    let mut current = Some(selected_id.to_string());
    while let Some(id) = current.take() {
        if !aliases.insert(id.clone()) {
            break;
        }
        if let Some(raw) = parse_snapshot_node_raw(&id) {
            aliases.insert(format!("node#{raw}"));
        }
        current = snapshot
            .and_then(|snapshot| snapshot.elements.get(&id))
            .and_then(|element| element.parent.clone());
    }
    aliases
}

fn parse_snapshot_node_raw(id: &str) -> Option<u64> {
    if let Some(raw) = id.strip_prefix("node#") {
        return raw.parse().ok();
    }
    id.strip_prefix("LayoutNodeId(")
        .and_then(|rest| rest.strip_suffix(')'))
        .and_then(|raw| raw.parse().ok())
}

fn make_state_callback0<F>(app_state: &SharedAppState, action: F) -> Arc<dyn Fn() + Send + Sync>
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let shared = app_state.clone();
    Arc::new(move || {
        if let Ok(mut state) = shared.write() {
            action(&mut state);
        }
    })
}

#[cfg(unix)]
#[derive(Debug)]
enum ConnectTarget {
    Unix(String),
    Tcp(String),
}

#[cfg(unix)]
fn resolve_connect_target(addr: &str) -> Result<ConnectTarget> {
    use std::net::{SocketAddr, ToSocketAddrs};

    if let Some(path) = addr.strip_prefix("unix:") {
        if path.is_empty() {
            bail!("unix target cannot be empty (use unix:/path/to/socket.sock)");
        }
        return Ok(ConnectTarget::Unix(path.to_string()));
    }
    if let Some(target) = addr.strip_prefix("tcp:") {
        if target.is_empty() {
            bail!("tcp target cannot be empty (use tcp:host:port)");
        }
        return Ok(ConnectTarget::Tcp(target.to_string()));
    }
    if addr.contains('/') || addr.contains('\\') {
        bail!(
            "path-like connect target '{addr}' is not allowed without scheme; use unix:/path.sock"
        );
    }

    if addr.parse::<SocketAddr>().is_ok() || (addr.contains(':') && addr.to_socket_addrs().is_ok())
    {
        return Ok(ConnectTarget::Tcp(addr.to_string()));
    }

    if !is_valid_default_socket_name(addr) {
        bail!(
            "invalid app name '{addr}'; allowed characters for default socket mapping are [A-Za-z0-9._-]"
        );
    }

    Ok(ConnectTarget::Unix(format!("/tmp/blinc/{addr}.sock")))
}

#[cfg(unix)]
fn is_valid_default_socket_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

#[cfg(unix)]
fn request_export_over_unix_socket(socket: &str) -> Result<RecordingExport> {
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    request_export_over_stream(&mut stream)
}

fn request_export_over_tcp(addr: &str) -> Result<RecordingExport> {
    let mut stream = std::net::TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    request_export_over_stream(&mut stream)
}

fn request_export_over_stream<S: Read + Write>(stream: &mut S) -> Result<RecordingExport> {
    let mut total_payload_bytes = 0usize;

    let hello_payload = read_len_prefixed(stream)?;
    add_payload_budget(&mut total_payload_bytes, hello_payload.len())?;
    let hello: serde_json::Value = serde_json::from_slice(&hello_payload)?;
    if hello.get("type").and_then(|v| v.as_str()) != Some("hello") {
        bail!("unexpected first server message: {hello}");
    }

    let request = serde_json::json!({ "type": "request_export" });
    let bytes = serde_json::to_vec(&request)?;
    write_len_prefixed(stream, &bytes)?;

    for _ in 0..MAX_SERVER_MESSAGES_TO_PARSE {
        let payload = read_len_prefixed(stream)?;
        add_payload_budget(&mut total_payload_bytes, payload.len())?;
        let value: serde_json::Value = serde_json::from_slice(&payload)?;
        match value.get("type").and_then(|v| v.as_str()) {
            Some("export") => {
                let export_value = value
                    .get("export")
                    .cloned()
                    .ok_or_else(|| anyhow!("missing export field in server response"))?;
                return serde_json::from_value(export_value).map_err(Into::into);
            }
            Some("error") => {
                let message = value
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown server error");
                bail!("server error: {message}");
            }
            _ => continue,
        }
    }

    bail!("did not receive export payload from server")
}

fn add_payload_budget(total_payload_bytes: &mut usize, payload_len: usize) -> Result<()> {
    let updated = total_payload_bytes
        .checked_add(payload_len)
        .ok_or_else(|| anyhow!("payload size overflow"))?;
    if updated > MAX_EXPORT_STREAM_PAYLOAD_BYTES {
        bail!(
            "total payload size {} exceeds maximum allowed {}",
            updated,
            MAX_EXPORT_STREAM_PAYLOAD_BYTES
        );
    }
    *total_payload_bytes = updated;
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::{
        add_payload_budget, read_len_prefixed, resolve_connect_target, AppState, ConnectTarget,
        SelectedTraceContext, MAX_EXPORT_STREAM_PAYLOAD_BYTES, MAX_NETWORK_PAYLOAD_BYTES,
    };
    use blinc_recorder::trace::{
        TraceArtifactRecord, TraceAssertionRecord, TraceCommandRecord, TraceLocatorResolution,
    };
    use blinc_recorder::{
        capture::Rect, ElementSnapshot, Key, KeyEvent, Modifiers, MouseMoveEvent, Point,
        RecordedEvent, RecordingConfig, RecordingExport, SessionStats, Timestamp, TimestampedEvent,
        TraceEntry, TraceEntryKind, TreeSnapshot,
    };
    use std::io::Cursor;

    #[test]
    fn resolves_unix_scheme_address() {
        assert!(matches!(
            resolve_connect_target("unix:/tmp/blinc/test.sock").expect("valid unix target"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/test.sock"
        ));
    }

    #[test]
    fn resolves_tcp_scheme_address() {
        assert!(matches!(
            resolve_connect_target("tcp:127.0.0.1:7331").expect("valid tcp target"),
            ConnectTarget::Tcp(target) if target == "127.0.0.1:7331"
        ));
    }

    #[test]
    fn resolves_unix_socket_path_with_unix_scheme() {
        assert!(matches!(
            resolve_connect_target("unix:/tmp/blinc/custom.sock").expect("valid unix target"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/custom.sock"
        ));
    }

    #[test]
    fn resolves_ip_socket_addr_as_tcp() {
        assert!(matches!(
            resolve_connect_target("127.0.0.1:7331").expect("valid tcp addr"),
            ConnectTarget::Tcp(target) if target == "127.0.0.1:7331"
        ));
    }

    #[test]
    fn resolves_app_name_to_default_socket_path() {
        assert!(matches!(
            resolve_connect_target("my_app").expect("valid app name"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/my_app.sock"
        ));
    }

    #[test]
    fn app_name_without_colon_stays_unix_target() {
        assert!(matches!(
            resolve_connect_target("example.com").expect("valid app name"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/example.com.sock"
        ));
    }

    #[test]
    fn rejects_path_like_target_without_scheme() {
        let err = resolve_connect_target("/tmp/blinc/custom.sock")
            .expect_err("path-like target without unix: prefix must be rejected");
        assert!(
            err.to_string().contains("path-like connect target"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_invalid_default_socket_name() {
        let err = resolve_connect_target("bad$name")
            .expect_err("app name with invalid characters must be rejected");
        assert!(
            err.to_string().contains("invalid app name"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_payload_larger_than_limit() {
        let len = (MAX_NETWORK_PAYLOAD_BYTES as u32) + 1;
        let mut frame = Vec::new();
        frame.extend_from_slice(&len.to_le_bytes());
        let mut cursor = Cursor::new(frame);

        let err = read_len_prefixed(&mut cursor).expect_err("oversized payload must be rejected");
        let expected = format!(
            "payload size {} exceeds maximum allowed {}",
            MAX_NETWORK_PAYLOAD_BYTES + 1,
            MAX_NETWORK_PAYLOAD_BYTES
        );
        assert!(
            err.to_string().contains(&expected),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn payload_budget_rejects_total_over_limit() {
        let mut total = MAX_EXPORT_STREAM_PAYLOAD_BYTES - 1;
        let err = add_payload_budget(&mut total, 2).expect_err("payload budget overflow expected");
        assert!(
            err.to_string().contains("total payload size"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn debugger_reads_enriched_recording_export_and_shows_command_stream() {
        let mut state = AppState::default();
        state.load_from_server("test", sample_recording_export());
        state.seek_normalized(1.0);

        let command_lines = state.command_stream_lines();
        let evidence_lines = state.evidence_lines();

        assert!(
            command_lines
                .iter()
                .any(|line| line == "fill -> login.submit [value=<redacted:17 chars>]"),
            "expected command stream to include click command: {command_lines:?}"
        );
        assert!(
            evidence_lines
                .iter()
                .any(|line| line.contains("FAIL assert_text_contains -> status")),
            "expected assertion evidence: {evidence_lines:?}"
        );
        assert!(
            evidence_lines
                .iter()
                .any(|line| line
                    .contains("artifact trace_export: wrote trace @ /tmp/blinc/trace.json")),
            "expected artifact path in evidence lines: {evidence_lines:?}"
        );
    }

    #[test]
    fn debugger_trace_panels_follow_replay_position() {
        let mut state = AppState::default();
        state.load_from_server("test", sample_recording_export());

        assert!(
            state.command_stream_lines().is_empty(),
            "expected command stream to stay empty before the first trace timestamp"
        );
        assert!(
            state.current_snapshot.is_none(),
            "expected no snapshot to be selected before the first snapshot timestamp"
        );
        assert!(
            state.selected_element_id.is_none(),
            "expected no selected element before the first snapshot timestamp"
        );

        state.seek_normalized(1.0);
        assert!(
            !state.command_stream_lines().is_empty(),
            "expected command stream to populate after seeking through the trace"
        );
    }

    #[test]
    fn inspector_shows_locator_resolution_and_assertion_context() {
        let mut state = AppState::default();
        state.load_from_server("test", sample_recording_export());
        state.seek_normalized(1.0);
        state.set_selected_element_id(Some("status".to_string()));

        let SelectedTraceContext {
            locator_lines,
            assertion_lines,
        } = state.selected_trace_context();

        assert!(
            locator_lines
                .iter()
                .any(|line| line.contains("role=Button") && line.contains("matched")),
            "expected locator context for selected element: {locator_lines:?}"
        );
        assert!(
            assertion_lines
                .iter()
                .any(|line| line.contains("FAIL assert_text_contains")),
            "expected assertion context for selected element: {assertion_lines:?}"
        );
        let element = state
            .selected_element()
            .expect("selected element should exist");
        assert_eq!(element.element_type, "button");
    }

    #[test]
    fn debugger_cursor_tracks_replay_position() {
        let mut state = AppState::default();
        state.load_from_server("test", sample_recording_export());

        assert_eq!(state.cursor_position, None);

        state.seek_normalized(1.0);
        assert_eq!(state.cursor_position, Some((48.0, 72.0)));

        state.seek_normalized(0.0);
        assert_eq!(
            state.cursor_position, None,
            "expected cursor to clear when scrubbing before the first event"
        );
    }

    #[test]
    fn debugger_cursor_keeps_last_pointer_position_after_non_pointer_events() {
        let mut export = sample_recording_export();
        export.events.push(TimestampedEvent {
            timestamp: Timestamp::from_micros(25),
            event: RecordedEvent::KeyDown(KeyEvent {
                key: Key::Enter,
                modifiers: Modifiers::default(),
                is_repeat: false,
                focused_element: Some("status".to_string()),
            }),
        });

        let mut state = AppState::default();
        state.load_from_server("test", export);
        state.seek_normalized(1.0);

        assert_eq!(
            state.cursor_position,
            Some((48.0, 72.0)),
            "expected cursor to keep the last known pointer position"
        );
    }

    #[test]
    fn debugger_cursor_tracks_hover_leave_position() {
        let mut export = sample_recording_export();
        export.events.push(TimestampedEvent {
            timestamp: Timestamp::from_micros(25),
            event: RecordedEvent::HoverLeave(blinc_recorder::HoverEvent {
                position: Point::new(64.0, 96.0),
                element_id: "status".to_string(),
            }),
        });

        let mut state = AppState::default();
        state.load_from_server("test", export);
        state.seek_normalized(1.0);

        assert_eq!(
            state.cursor_position,
            Some((64.0, 96.0)),
            "expected hover leave to keep the last pointer location visible"
        );
    }

    #[test]
    fn inspector_marks_non_selected_locator_candidates_as_candidates() {
        let mut export = sample_recording_export();
        export.trace_entries[1].kind = TraceEntryKind::LocatorResolution(TraceLocatorResolution {
            query: "role=Button".to_string(),
            matched_target: Some("other-button".to_string()),
            candidate_targets: vec!["status".to_string(), "other-button".to_string()],
            failure_reason: None,
        });

        let mut state = AppState::default();
        state.load_from_server("test", export);
        state.seek_normalized(1.0);
        state.set_selected_element_id(Some("status".to_string()));

        let context = state.selected_trace_context();
        assert!(
            context
                .locator_lines
                .iter()
                .any(|line| line.contains("[candidate]")),
            "expected non-selected locator candidates to be labeled as candidates: {:?}",
            context.locator_lines
        );
    }

    #[test]
    fn inspector_matches_trace_context_for_selected_idless_descendants() {
        let mut export = sample_recording_export();
        export.snapshots[0].elements.insert(
            "node#42".to_string(),
            ElementSnapshot {
                id: "node#42".to_string(),
                element_type: "text".to_string(),
                bounds: Rect {
                    x: 24.0,
                    y: 32.0,
                    width: 72.0,
                    height: 18.0,
                },
                parent: Some("status".to_string()),
                children: Vec::new(),
                is_visible: true,
                is_focused: false,
                is_hovered: false,
                is_interactive: false,
                text_content: Some("Ready".to_string()),
                visual_props: None,
            },
        );
        export.snapshots[0]
            .elements
            .get_mut("status")
            .expect("status element should exist")
            .children
            .push("node#42".to_string());

        let mut state = AppState::default();
        state.load_from_server("test", export);
        state.seek_normalized(1.0);
        state.set_selected_element_id(Some("node#42".to_string()));

        let context = state.selected_trace_context();
        assert!(
            context
                .locator_lines
                .iter()
                .any(|line| line.contains("role=Button") && line.contains("[matched]")),
            "expected locator context for selected id-less descendant: {:?}",
            context.locator_lines
        );
        assert!(
            context
                .assertion_lines
                .iter()
                .any(|line| line.contains("assert_text_contains")),
            "expected assertion context for selected id-less descendant: {:?}",
            context.assertion_lines
        );
    }

    #[test]
    fn debugger_evidence_surfaces_targetless_locator_failures() {
        let mut export = sample_recording_export();
        export.trace_entries.push(TraceEntry {
            sequence: 4,
            timestamp: Timestamp::from_micros(13),
            kind: TraceEntryKind::LocatorResolution(TraceLocatorResolution {
                query: "role=Button, text=\"Missing\"".to_string(),
                matched_target: None,
                candidate_targets: vec!["status".to_string(), "other".to_string()],
                failure_reason: Some("ambiguous_match".to_string()),
            }),
        });

        let mut state = AppState::default();
        state.load_from_server("test", export);
        state.seek_normalized(1.0);

        let evidence_lines = state.evidence_lines();
        assert!(
            evidence_lines.iter().any(|line| {
                line.contains("locator role=Button, text=\"Missing\"")
                    && line.contains("[ambiguous_match]")
                    && line.contains("candidates=status, other")
            }),
            "expected targetless locator failure to appear in evidence: {evidence_lines:?}"
        );
    }

    fn sample_recording_export() -> RecordingExport {
        let mut snapshot = TreeSnapshot::new(Timestamp::from_micros(10), (400, 300), 1.0);
        snapshot.root_id = Some("root".to_string());
        snapshot.elements.insert(
            "root".to_string(),
            ElementSnapshot {
                id: "root".to_string(),
                element_type: "div".to_string(),
                bounds: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 400.0,
                    height: 300.0,
                },
                parent: None,
                children: vec!["status".to_string()],
                is_visible: true,
                is_focused: false,
                is_hovered: false,
                is_interactive: false,
                text_content: None,
                visual_props: None,
            },
        );
        snapshot.elements.insert(
            "status".to_string(),
            ElementSnapshot {
                id: "status".to_string(),
                element_type: "button".to_string(),
                bounds: Rect {
                    x: 16.0,
                    y: 24.0,
                    width: 120.0,
                    height: 32.0,
                },
                parent: Some("root".to_string()),
                children: Vec::new(),
                is_visible: true,
                is_focused: false,
                is_hovered: false,
                is_interactive: true,
                text_content: Some("Ready".to_string()),
                visual_props: None,
            },
        );

        RecordingExport {
            schema_version: blinc_recorder::session::RECORDING_EXPORT_VERSION,
            config: RecordingConfig::minimal(),
            events: vec![TimestampedEvent {
                timestamp: Timestamp::from_micros(20),
                event: RecordedEvent::MouseMove(MouseMoveEvent {
                    position: Point::new(48.0, 72.0),
                    hover_element: Some("status".to_string()),
                }),
            }],
            snapshots: vec![snapshot],
            trace_entries: vec![
                TraceEntry {
                    sequence: 1,
                    timestamp: Timestamp::from_micros(10),
                    kind: TraceEntryKind::Command(TraceCommandRecord {
                        name: "fill".to_string(),
                        target: Some("login.submit".to_string()),
                        payload: Some("value=<redacted:17 chars>".to_string()),
                    }),
                },
                TraceEntry {
                    sequence: 2,
                    timestamp: Timestamp::from_micros(11),
                    kind: TraceEntryKind::LocatorResolution(TraceLocatorResolution {
                        query: "role=Button".to_string(),
                        matched_target: Some("status".to_string()),
                        candidate_targets: vec!["status".to_string()],
                        failure_reason: None,
                    }),
                },
                TraceEntry {
                    sequence: 3,
                    timestamp: Timestamp::from_micros(12),
                    kind: TraceEntryKind::Assertion(TraceAssertionRecord {
                        code: "assert_text_contains".to_string(),
                        passed: false,
                        target: Some("status".to_string()),
                        actual: Some("Ready".to_string()),
                        expected: Some("Signed in".to_string()),
                    }),
                },
                TraceEntry {
                    sequence: 4,
                    timestamp: Timestamp::from_micros(13),
                    kind: TraceEntryKind::Artifact(TraceArtifactRecord {
                        kind: "trace_export".to_string(),
                        path: Some("/tmp/blinc/trace.json".to_string()),
                        message: Some("wrote trace".to_string()),
                    }),
                },
            ],
            stats: SessionStats::default(),
        }
    }
}
