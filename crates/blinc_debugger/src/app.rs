//! Main application module for the debugger.

use crate::panels::{
    InspectorPanel, PreviewConfig, PreviewPanel, TimelinePanel, TimelinePanelState, TreePanel,
    TreePanelState,
};
use crate::theme::DebuggerColors;
use anyhow::{anyhow, bail, Context, Result};
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_app::WindowConfig;
use blinc_layout::prelude::*;
use blinc_recorder::replay::{
    FrameUpdate, ReplayConfig, ReplayPlayer, ReplayState, SimulatedInput,
};
use blinc_recorder::{RecordingExport, Timestamp, TreeSnapshot};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

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
            } else {
                self.current_snapshot = player
                    .all_snapshots()
                    .iter()
                    .rfind(|s| s.timestamp <= player.position())
                    .cloned()
                    .or_else(|| self.current_snapshot.take());
            }

            self.timeline_state.position = player.position();
            self.timeline_state.duration = player.duration();
            self.timeline_state.playback_state = player.state();
            self.timeline_state.speed = player.clock().speed();
        }

        self.ensure_selected_element_exists();
    }

    pub fn toggle_playback(&mut self) {
        let player_arc = self.player.clone();
        if let Some(player_arc) = player_arc {
            if let Ok(mut player) = player_arc.lock() {
                player.toggle();
                self.timeline_state.playback_state = player.state();
            }
        }
    }

    pub fn step_back(&mut self) {
        let player_arc = self.player.clone();
        if let Some(player_arc) = player_arc {
            if let Ok(mut player) = player_arc.lock() {
                let update = player.step_back();
                self.apply_frame_update(update);
                self.timeline_state.position = player.position();
                self.timeline_state.playback_state = player.state();
            }
        }
    }

    pub fn step_forward(&mut self) {
        let player_arc = self.player.clone();
        if let Some(player_arc) = player_arc {
            if let Ok(mut player) = player_arc.lock() {
                let update = player.step();
                self.apply_frame_update(update);
                self.timeline_state.position = player.position();
                self.timeline_state.playback_state = player.state();
            }
        }
    }

    pub fn seek_normalized(&mut self, normalized: f32) {
        let player_arc = self.player.clone();
        if let Some(player_arc) = player_arc {
            if let Ok(mut player) = player_arc.lock() {
                let micros = (player.duration().as_micros() as f32 * normalized.clamp(0.0, 1.0))
                    .round() as u64;
                player.seek(Timestamp::from_micros(micros));
                self.timeline_state.position = player.position();
                self.timeline_state.playback_state = ReplayState::Paused;
                self.current_snapshot = player
                    .all_snapshots()
                    .iter()
                    .rfind(|s| s.timestamp <= player.position())
                    .cloned();
            }
        }
    }

    pub fn set_playback_speed(&mut self, speed: f64) {
        let player_arc = self.player.clone();
        if let Some(player_arc) = player_arc {
            if let Ok(mut player) = player_arc.lock() {
                player.clock_mut().set_speed(speed);
                self.timeline_state.speed = player.clock().speed();
            }
        }
    }

    fn apply_recording(&mut self, export: RecordingExport) {
        let player = ReplayPlayer::new(export.clone(), ReplayConfig::interactive());
        self.timeline_state.duration = player.duration();
        self.timeline_state.position = Timestamp::zero();
        self.timeline_state.playback_state = ReplayState::Idle;
        self.timeline_state.speed = player.clock().speed();

        self.current_snapshot = export.snapshots.first().cloned();
        self.selected_element_id = self.current_snapshot.as_ref().and_then(|s| {
            s.root_id
                .clone()
                .or_else(|| s.elements.keys().next().cloned())
        });
        self.cursor_position = None;

        self.recording = Some(export);
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
            self.selected_element_id = None;
            return;
        };

        match self.selected_element_id.as_ref() {
            Some(id) if snapshot.elements.contains_key(id) => {}
            _ => {
                self.selected_element_id = snapshot
                    .root_id
                    .clone()
                    .or_else(|| snapshot.elements.keys().next().cloned());
            }
        }
    }
}

/// Shared application state for thread-safe access.
pub type SharedAppState = Arc<RwLock<AppState>>;

/// Run the debugger application.
pub fn run(width: u32, height: u32, file: Option<PathBuf>, connect: Option<String>) -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::default()));

    if let Some(ref path) = file {
        if let Err(e) = app_state.write().unwrap().load_recording(path) {
            log::warn!("Failed to load recording from {:?}: {}", path, e);
        }
    }

    if let Some(ref addr) = connect {
        match request_export_from_server(addr) {
            Ok(export) => app_state.write().unwrap().load_from_server(addr, export),
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
    app_state.write().unwrap().tick();
    let state = app_state.read().unwrap();

    let on_tree_select = {
        let shared = app_state.clone();
        Arc::new(move |id: String| {
            if let Ok(mut state) = shared.write() {
                state.selected_element_id = Some(id);
            }
        })
    };

    let on_toggle_bounds = {
        let shared = app_state.clone();
        Arc::new(move |value: bool| {
            if let Ok(mut state) = shared.write() {
                state.preview_config.show_bounds = value;
            }
        })
    };

    let on_toggle_cursor = {
        let shared = app_state.clone();
        Arc::new(move |value: bool| {
            if let Ok(mut state) = shared.write() {
                state.preview_config.show_cursor = value;
            }
        })
    };

    let on_zoom = {
        let shared = app_state.clone();
        Arc::new(move |value: f32| {
            if let Ok(mut state) = shared.write() {
                state.preview_config.zoom = value;
            }
        })
    };

    let on_step_back = {
        let shared = app_state.clone();
        Arc::new(move || {
            if let Ok(mut state) = shared.write() {
                state.step_back();
            }
        })
    };

    let on_play_pause = {
        let shared = app_state.clone();
        Arc::new(move || {
            if let Ok(mut state) = shared.write() {
                state.toggle_playback();
            }
        })
    };

    let on_step_forward = {
        let shared = app_state.clone();
        Arc::new(move || {
            if let Ok(mut state) = shared.write() {
                state.step_forward();
            }
        })
    };

    let on_seek = {
        let shared = app_state.clone();
        Arc::new(move |normalized: f32| {
            if let Ok(mut state) = shared.write() {
                state.seek_normalized(normalized);
            }
        })
    };

    let on_speed_change = {
        let shared = app_state.clone();
        Arc::new(move |speed: f64| {
            if let Ok(mut state) = shared.write() {
                state.set_playback_speed(speed);
            }
        })
    };

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
                .child(PreviewPanel::new(
                    state.current_snapshot.as_ref(),
                    &state.preview_config,
                    state.cursor_position,
                    Some(on_toggle_bounds),
                    Some(on_toggle_cursor),
                    Some(on_zoom),
                ))
                .child(InspectorPanel::new(state.selected_element())),
        )
        .child(TimelinePanel::new(
            state
                .recording
                .as_ref()
                .map(|r| r.events.as_slice())
                .unwrap_or(&[]),
            &state.timeline_state,
            Some(on_step_back),
            Some(on_play_pause),
            Some(on_step_forward),
            Some(on_seek),
            Some(on_speed_change),
        ))
}

fn read_len_prefixed<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
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
        if !addr.contains(':') || addr.contains('/') {
            let socket = if addr.contains('/') {
                addr.to_string()
            } else {
                format!("/tmp/blinc/{addr}.sock")
            };
            use std::os::unix::net::UnixStream;
            let mut stream = UnixStream::connect(&socket)
                .with_context(|| format!("failed to connect to unix socket {socket}"))?;
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
            return request_export_over_stream(&mut stream);
        }
    }

    let mut stream = std::net::TcpStream::connect(addr)
        .with_context(|| format!("failed to connect to tcp server {addr}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    request_export_over_stream(&mut stream)
}

fn request_export_over_stream<S: Read + Write>(stream: &mut S) -> Result<RecordingExport> {
    let hello_payload = read_len_prefixed(stream)?;
    let hello: serde_json::Value = serde_json::from_slice(&hello_payload)?;
    if hello.get("type").and_then(|v| v.as_str()) != Some("hello") {
        bail!("unexpected first server message: {hello}");
    }

    let request = serde_json::json!({ "type": "request_export" });
    let bytes = serde_json::to_vec(&request)?;
    write_len_prefixed(stream, &bytes)?;

    for _ in 0..32 {
        let payload = read_len_prefixed(stream)?;
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
