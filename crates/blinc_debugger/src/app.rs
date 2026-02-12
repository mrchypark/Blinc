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
                if let Some(snapshot) = player
                    .all_snapshots()
                    .iter()
                    .rfind(|s| s.timestamp <= player.position())
                    .cloned()
                {
                    self.current_snapshot = Some(snapshot);
                }
            }

            self.timeline_state.position = player.position();
            self.timeline_state.duration = player.duration();
            self.timeline_state.playback_state = player.state();
            self.timeline_state.speed = player.clock().speed();
        }

        self.ensure_selected_element_exists();
    }

    pub fn toggle_playback(&mut self) {
        self.with_player(|state, player| {
            player.toggle();
            state.timeline_state.playback_state = player.state();
        });
    }

    pub fn step_back(&mut self) {
        self.with_player(|state, player| {
            let update = player.step_back();
            state.apply_frame_update(update);
            state.timeline_state.position = player.position();
            state.timeline_state.playback_state = player.state();
        });
    }

    pub fn step_forward(&mut self) {
        self.with_player(|state, player| {
            let update = player.step();
            state.apply_frame_update(update);
            state.timeline_state.position = player.position();
            state.timeline_state.playback_state = player.state();
        });
    }

    pub fn seek_normalized(&mut self, normalized: f32) {
        self.with_player(|state, player| {
            let micros =
                (player.duration().as_micros() as f32 * normalized.clamp(0.0, 1.0)).round() as u64;
            player.seek(Timestamp::from_micros(micros));
            state.timeline_state.position = player.position();
            state.timeline_state.playback_state = ReplayState::Paused;
            state.current_snapshot = player
                .all_snapshots()
                .iter()
                .rfind(|s| s.timestamp <= player.position())
                .cloned();
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
        state.selected_element_id = Some(id);
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
        match resolve_connect_target(addr) {
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
enum ConnectTarget {
    Unix(String),
    Tcp(String),
}

#[cfg(unix)]
fn resolve_connect_target(addr: &str) -> ConnectTarget {
    use std::net::{SocketAddr, ToSocketAddrs};

    if let Some(path) = addr.strip_prefix("unix:") {
        return ConnectTarget::Unix(path.to_string());
    }
    if let Some(target) = addr.strip_prefix("tcp:") {
        return ConnectTarget::Tcp(target.to_string());
    }
    if addr.contains('/') {
        return ConnectTarget::Unix(addr.to_string());
    }

    if addr.parse::<SocketAddr>().is_ok() || (addr.contains(':') && addr.to_socket_addrs().is_ok())
    {
        return ConnectTarget::Tcp(addr.to_string());
    }

    ConnectTarget::Unix(format!("/tmp/blinc/{addr}.sock"))
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
        add_payload_budget, read_len_prefixed, resolve_connect_target, ConnectTarget,
        MAX_EXPORT_STREAM_PAYLOAD_BYTES, MAX_NETWORK_PAYLOAD_BYTES,
    };
    use std::io::Cursor;

    #[test]
    fn resolves_unix_scheme_address() {
        assert!(matches!(
            resolve_connect_target("unix:/tmp/blinc/test.sock"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/test.sock"
        ));
    }

    #[test]
    fn resolves_tcp_scheme_address() {
        assert!(matches!(
            resolve_connect_target("tcp:127.0.0.1:7331"),
            ConnectTarget::Tcp(target) if target == "127.0.0.1:7331"
        ));
    }

    #[test]
    fn resolves_unix_socket_path() {
        assert!(matches!(
            resolve_connect_target("/tmp/blinc/custom.sock"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/custom.sock"
        ));
    }

    #[test]
    fn resolves_ip_socket_addr_as_tcp() {
        assert!(matches!(
            resolve_connect_target("127.0.0.1:7331"),
            ConnectTarget::Tcp(target) if target == "127.0.0.1:7331"
        ));
    }

    #[test]
    fn resolves_app_name_to_default_socket_path() {
        assert!(matches!(
            resolve_connect_target("my_app"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/my_app.sock"
        ));
    }

    #[test]
    fn app_name_without_colon_stays_unix_target() {
        assert!(matches!(
            resolve_connect_target("example.com"),
            ConnectTarget::Unix(path) if path == "/tmp/blinc/example.com.sock"
        ));
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
}
