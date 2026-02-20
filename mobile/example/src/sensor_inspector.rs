use blinc_app::prelude::*;
use blinc_app::windowed::WindowedContext;
use blinc_core::native_bridge::native_call;
use blinc_core::reactive::State;
use blinc_sensors::native_bridge::NativeBridgeBackend;
use blinc_sensors::{SensorClient, SensorConfig, SensorFrame, SensorKind};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const SENSOR_SESSION_ID: &str = "blinc-example-live";
const SENSOR_POLL_INTERVAL_SECS: f32 = 0.35;

#[derive(Clone)]
pub struct SensorPanelData {
    pub status_line: String,
    pub permission_line: String,
    pub supported_line: String,
    pub kinds_line: String,
    pub sample_line: String,
    pub accel_line: String,
    pub gyro_line: String,
    pub magnet_line: String,
    pub barometer_line: String,
    pub step_line: String,
    pub activity_line: String,
    pub note_line: String,
}

impl Default for SensorPanelData {
    fn default() -> Self {
        Self {
            status_line: "running=false  session=-  buffered=0".to_string(),
            permission_line: "permission: location=false  motion=false".to_string(),
            supported_line: "supported: []".to_string(),
            kinds_line: "kinds: []".to_string(),
            sample_line: "sample: turn ON to stream frames".to_string(),
            accel_line: "accelerometer: no data".to_string(),
            gyro_line: "gyroscope: no data".to_string(),
            magnet_line: "magnetometer: no data".to_string(),
            barometer_line: "barometer: no data".to_string(),
            step_line: "steps/cadence/floors: no data".to_string(),
            activity_line: "activity: no data".to_string(),
            note_line: "source: blinc_sensors::NativeBridgeBackend".to_string(),
        }
    }
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn sensor_client() -> SensorClient<NativeBridgeBackend> {
    SensorClient::new(NativeBridgeBackend)
}

fn sensor_config() -> SensorConfig {
    SensorConfig {
        enabled: vec![
            SensorKind::Gps,
            SensorKind::Heading,
            SensorKind::Accelerometer,
            SensorKind::Gyroscope,
            SensorKind::Magnetometer,
            SensorKind::Barometer,
            SensorKind::StepCounter,
            SensorKind::Cadence,
            SensorKind::FloorClimb,
            SensorKind::Activity,
        ],
        gps_hz: 1,
        imu_hz: 25,
        frame_flush_ms: 200,
    }
}

fn sensor_permission_state() -> (bool, bool) {
    let has_location = native_call::<bool, _>("permissions", "has_location", ()).unwrap_or(false);
    let has_motion = native_call::<bool, _>("permissions", "has_motion", ()).unwrap_or(false);
    (has_location, has_motion)
}

fn prepare_sensor_streaming(client: &SensorClient<NativeBridgeBackend>) -> std::result::Result<(), String> {
    let _ = native_call::<bool, _>("permissions", "request_location_when_in_use", ());
    let _ = native_call::<bool, _>("permissions", "request_motion", ());

    client
        .configure(&sensor_config())
        .map_err(|err| format!("configure failed: {}", err))?;

    let _ = native_call::<(), _>("sensor", "clear_buffer", ());
    Ok(())
}

fn start_sensor_streaming(client: &SensorClient<NativeBridgeBackend>) -> std::result::Result<(), String> {
    client
        .start_session(SENSOR_SESSION_ID)
        .map_err(|err| format!("start failed: {}", err))?;

    let status = client
        .status()
        .map_err(|err| format!("status failed: {}", err))?;

    if !status.running {
        return Err(
            "sensor.start succeeded but status.running=false (check Location Services / permission)"
                .to_string(),
        );
    }

    Ok(())
}

fn stop_sensor_streaming(client: &SensorClient<NativeBridgeBackend>) -> std::result::Result<(), String> {
    client
        .stop_session(SENSOR_SESSION_ID)
        .map_err(|err| format!("stop failed: {}", err))?;
    Ok(())
}

fn summarize_frames(frames: &[SensorFrame]) -> (BTreeMap<SensorKind, usize>, BTreeMap<SensorKind, String>) {
    let mut counts: BTreeMap<SensorKind, usize> = BTreeMap::new();
    let mut last_samples: BTreeMap<SensorKind, String> = BTreeMap::new();

    for frame in frames {
        *counts.entry(frame.sensor).or_insert(0) += 1;
        let compact = frame
            .values
            .iter()
            .take(4)
            .map(|v| format!("{:.3}", v))
            .collect::<Vec<_>>()
            .join(", ");
        if !compact.is_empty() {
            last_samples.insert(frame.sensor, compact);
        }
    }

    (counts, last_samples)
}

fn fetch_sensor_panel_data(client: &SensorClient<NativeBridgeBackend>) -> SensorPanelData {
    let mut data = SensorPanelData::default();

    let (has_location, has_motion) = sensor_permission_state();
    data.permission_line = format!(
        "permission: location={}  motion={}",
        has_location, has_motion
    );

    let status = match client.status() {
        Ok(status) => status,
        Err(err) => {
            data.note_line = format!("sensor unavailable: {}", err);
            return data;
        }
    };

    data.status_line = format!(
        "running={}  session={}  buffered={}",
        status.running,
        status
            .active_session_id
            .as_deref()
            .unwrap_or("-"),
        status.buffered_frames
    );

    if let Ok(kinds) = client.supported_kinds() {
        if kinds.is_empty() {
            data.supported_line = "supported: []".to_string();
        } else {
            let names: Vec<&'static str> = kinds.into_iter().map(SensorKind::as_str).collect();
            data.supported_line = format!(
                "supported: [{}]",
                names.join(", ")
            );
        }
    }

    let frames = match client.drain_frames(256) {
        Ok(frames) => frames,
        Err(err) => {
            data.note_line = format!("drain failed: {}", err);
            return data;
        }
    };

    if frames.is_empty() {
        data.sample_line = "sample: no frames yet (move device / change location)".to_string();
        data.note_line = format!("auto poll captured 0 frames @{}ms", now_unix_ms());
        return data;
    }

    let (counts, last_samples) = summarize_frames(&frames);

    if counts.is_empty() {
        data.kinds_line = "kinds: []".to_string();
    } else {
        let summary = counts
            .iter()
            .map(|(kind, count)| format!("{}={}", kind.as_str(), count))
            .collect::<Vec<_>>()
            .join(", ");
        data.kinds_line = format!("kinds: [{}]", summary);
    }

    if let Some(sample) = frames.last() {
        let compact = sample
            .values
            .iter()
            .take(4)
            .map(|v| format!("{:.3}", v))
            .collect::<Vec<_>>()
            .join(", ");
        data.sample_line = format!("sample: {} [{}]", sample.sensor.as_str(), compact);
    }

    let line_for = |label: &str, kind: SensorKind| -> String {
        let count = counts.get(&kind).copied().unwrap_or(0);
        match last_samples.get(&kind) {
            Some(values) if !values.is_empty() => format!("{}: {} frames  [{}]", label, count, values),
            _ => format!("{}: {} frames  [no data]", label, count),
        }
    };

    data.accel_line = line_for("accelerometer", SensorKind::Accelerometer);
    data.gyro_line = line_for("gyroscope", SensorKind::Gyroscope);
    data.magnet_line = line_for("magnetometer", SensorKind::Magnetometer);
    data.barometer_line = line_for("barometer", SensorKind::Barometer);

    let step_count = counts.get(&SensorKind::StepCounter).copied().unwrap_or(0)
        + counts.get(&SensorKind::Cadence).copied().unwrap_or(0)
        + counts.get(&SensorKind::FloorClimb).copied().unwrap_or(0);

    let step_values = [
        ("steps", SensorKind::StepCounter),
        ("cadence", SensorKind::Cadence),
        ("floors", SensorKind::FloorClimb),
    ]
    .iter()
    .filter_map(|(name, kind)| {
        last_samples
            .get(kind)
            .map(|values| format!("{}=[{}]", name, values))
    })
    .collect::<Vec<_>>();

    data.step_line = if step_values.is_empty() {
        format!("steps/cadence/floors: {} frames  [no data]", step_count)
    } else {
        format!(
            "steps/cadence/floors: {} frames  {}",
            step_count,
            step_values.join("  ")
        )
    };

    data.activity_line = line_for("activity", SensorKind::Activity);
    data.note_line = format!("auto poll captured {} frames @{}ms", frames.len(), now_unix_ms());
    data
}

fn sensor_toggle_button(
    snapshot: State<SensorPanelData>,
    live_enabled: State<bool>,
    pending_start: State<bool>,
) -> impl ElementBuilder {
    let client = sensor_client();
    let live_state = live_enabled.clone();

    stateful::<ButtonState>()
        .deps([live_enabled.signal_id()])
        .on_state(move |ctx| {
            let is_live = live_state.get();
            let bg = match (ctx.state(), is_live) {
                (ButtonState::Idle, false) => Color::rgba(0.3, 0.3, 0.4, 1.0),
                (ButtonState::Hovered, false) => Color::rgba(0.4, 0.4, 0.5, 1.0),
                (ButtonState::Pressed, false) => Color::rgba(0.2, 0.2, 0.3, 1.0),
                (ButtonState::Idle, true) => Color::rgba(0.18, 0.38, 0.26, 1.0),
                (ButtonState::Hovered, true) => Color::rgba(0.22, 0.46, 0.31, 1.0),
                (ButtonState::Pressed, true) => Color::rgba(0.12, 0.28, 0.19, 1.0),
                (ButtonState::Disabled, _) => Color::rgba(0.2, 0.2, 0.2, 0.5),
            };

            div()
                .w(170.0)
                .h(42.0)
                .rounded(8.0)
                .bg(bg)
                .items_center()
                .justify_center()
                .cursor(CursorStyle::Pointer)
                .child(
                    text(if is_live { "Sensors ON" } else { "Sensors OFF" })
                        .size(16.0)
                        .weight(FontWeight::SemiBold)
                        .color(Color::WHITE),
                )
        })
        .on_click(move |_| {
            if live_enabled.get() {
                match stop_sensor_streaming(&client) {
                    Ok(()) => {
                        live_enabled.set(false);
                        pending_start.set(false);
                        let mut data = fetch_sensor_panel_data(&client);
                        data.note_line = "stream stopped (OFF)".to_string();
                        snapshot.set(data);
                    }
                    Err(err) => {
                        pending_start.set(false);
                        let mut data = fetch_sensor_panel_data(&client);
                        data.note_line = err;
                        snapshot.set(data);
                    }
                }
                return;
            }

            if let Err(err) = prepare_sensor_streaming(&client) {
                pending_start.set(false);
                live_enabled.set(false);
                let mut data = fetch_sensor_panel_data(&client);
                data.note_line = err;
                snapshot.set(data);
                return;
            }

            let (has_location, has_motion) = sensor_permission_state();
            if !(has_location && has_motion) {
                live_enabled.set(true);
                pending_start.set(true);
                let mut data = fetch_sensor_panel_data(&client);
                data.note_line = format!(
                    "waiting permissions: location={} motion={} (grant prompt, then auto-start)",
                    has_location, has_motion
                );
                snapshot.set(data);
                return;
            }

            match start_sensor_streaming(&client) {
                Ok(()) => {
                    live_enabled.set(true);
                    pending_start.set(false);
                    let mut data = fetch_sensor_panel_data(&client);
                    data.note_line = "stream running (ON): auto-updating".to_string();
                    snapshot.set(data);
                }
                Err(err) => {
                    pending_start.set(false);
                    live_enabled.set(false);
                    let mut data = fetch_sensor_panel_data(&client);
                    data.note_line = err;
                    snapshot.set(data);
                }
            }
        })
}

pub fn sensor_section(ctx: &WindowedContext, section_card: fn(&str) -> Div) -> Div {
    let snapshot = ctx.use_state_keyed("sensor_snapshot", SensorPanelData::default);
    let live_enabled = ctx.use_state_keyed("sensor_live_enabled", || false);
    let pending_start = ctx.use_state_keyed("sensor_pending_start", || false);

    ctx.use_tick_callback_for("sensor-live-poll", {
        let client = sensor_client();
        let live_enabled = live_enabled.clone();
        let pending_start = pending_start.clone();
        let snapshot = snapshot.clone();
        let mut accumulator = 0.0f32;
        move |dt| {
            if !live_enabled.get() {
                accumulator = 0.0;
                return;
            }

            accumulator += dt;
            if accumulator < SENSOR_POLL_INTERVAL_SECS {
                return;
            }

            let steps = (accumulator / SENSOR_POLL_INTERVAL_SECS).floor() as u64;
            accumulator -= SENSOR_POLL_INTERVAL_SECS * steps as f32;
            if steps > 0 {
                if pending_start.get() {
                    let status_running = client.status().map(|status| status.running).unwrap_or(false);
                    if status_running {
                        pending_start.set(false);
                        let mut data = fetch_sensor_panel_data(&client);
                        data.note_line = "stream running (ON): auto-updating".to_string();
                        snapshot.set(data);
                        return;
                    }

                    let (has_location, has_motion) = sensor_permission_state();
                    if has_location && has_motion {
                        match start_sensor_streaming(&client) {
                            Ok(()) => {
                                pending_start.set(false);
                                let mut data = fetch_sensor_panel_data(&client);
                                data.note_line = "stream running (ON): auto-updating".to_string();
                                snapshot.set(data);
                            }
                            Err(err) => {
                                pending_start.set(false);
                                live_enabled.set(false);
                                let mut data = fetch_sensor_panel_data(&client);
                                data.note_line = err;
                                snapshot.set(data);
                            }
                        }
                        return;
                    }

                    let mut data = fetch_sensor_panel_data(&client);
                    data.note_line = format!(
                        "waiting permissions: location={} motion={} (grant prompt, then auto-start)",
                        has_location, has_motion
                    );
                    snapshot.set(data);
                    return;
                }

                let data = fetch_sensor_panel_data(&client);
                snapshot.set(data);
            }
        }
    });

    let detail_snapshot = snapshot.clone();

    section_card("Sensor Inspector")
        .id("sensor-section")
        .child(
            text("Turn ON to stream continuously. OFF freezes the last snapshot.")
                .size(14.0)
                .color(Color::rgba(0.6, 0.6, 0.7, 1.0))
                .align(TextAlign::Center),
        )
        .child(sensor_toggle_button(
            snapshot.clone(),
            live_enabled.clone(),
            pending_start.clone(),
        ))
        .child(
            div()
                .w_full()
                .flex_col()
                .gap(4.0)
                .py(8.0)
                .px(10.0)
                .bg(Color::rgba(0.18, 0.18, 0.23, 1.0))
                .rounded(12.0)
                .items_start()
                .child(
                    text("Live Sensor Snapshot")
                        .size(14.0)
                        .weight(FontWeight::SemiBold)
                        .color(Color::WHITE),
                )
                .child(
                    stateful::<NoState>()
                        .deps([snapshot.signal_id(), live_enabled.signal_id()])
                        .on_state(move |_| {
                            let data = detail_snapshot.get();
                            div()
                                .w_full()
                                .flex_col()
                                .gap(2.0)
                                .items_start()
                                .child(
                                    text(data.status_line.clone())
                                        .size(12.0)
                                        .color(Color::rgba(0.85, 0.9, 1.0, 1.0)),
                                )
                                .child(
                                    text(data.permission_line.clone())
                                        .size(11.0)
                                        .color(Color::rgba(0.75, 0.82, 0.9, 1.0)),
                                )
                                .child(
                                    text(data.supported_line.clone())
                                        .size(11.0)
                                        .color(Color::rgba(0.75, 0.82, 0.9, 1.0)),
                                )
                                .child(
                                    text(data.kinds_line.clone())
                                        .size(11.0)
                                        .color(Color::rgba(0.75, 0.82, 0.9, 1.0)),
                                )
                                .child(
                                    text(data.sample_line.clone())
                                        .size(11.0)
                                        .color(Color::rgba(0.75, 0.82, 0.9, 1.0)),
                                )
                                .child(
                                    text(data.accel_line.clone())
                                        .size(10.5)
                                        .color(Color::rgba(0.70, 0.80, 0.88, 1.0)),
                                )
                                .child(
                                    text(data.gyro_line.clone())
                                        .size(10.5)
                                        .color(Color::rgba(0.70, 0.80, 0.88, 1.0)),
                                )
                                .child(
                                    text(data.magnet_line.clone())
                                        .size(10.5)
                                        .color(Color::rgba(0.70, 0.80, 0.88, 1.0)),
                                )
                                .child(
                                    text(data.barometer_line.clone())
                                        .size(10.5)
                                        .color(Color::rgba(0.70, 0.80, 0.88, 1.0)),
                                )
                                .child(
                                    text(data.step_line.clone())
                                        .size(10.5)
                                        .color(Color::rgba(0.70, 0.80, 0.88, 1.0)),
                                )
                                .child(
                                    text(data.activity_line.clone())
                                        .size(10.5)
                                        .color(Color::rgba(0.70, 0.80, 0.88, 1.0)),
                                )
                                .child(
                                    text(data.note_line.clone())
                                        .size(11.0)
                                        .color(Color::rgba(0.5, 0.8, 0.6, 1.0)),
                                )
                        }),
                ),
        )
}
