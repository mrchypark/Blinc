//! Timeline Panel - Event timeline with playback controls

use std::cell::OnceCell;
use std::sync::Arc;

use blinc_cn::components::button::{button, ButtonSize, ButtonVariant};
use blinc_cn::components::select::{select, SelectSize};
use blinc_cn::components::separator::separator;
use blinc_cn::components::slider::{slider, SliderSize};
use blinc_core::context_state::BlincContextState;
use blinc_icons::icons;
use blinc_layout::div::{Div, ElementBuilder};
use blinc_layout::element::RenderProps;
use blinc_layout::event_handler::EventHandlers;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_recorder::replay::ReplayState;
use blinc_recorder::{Timestamp, TimestampedEvent, TraceEntry, TraceEntryKind};
use blinc_theme::{ColorToken, ThemeState};

use crate::theme::DebuggerTokens;

#[derive(Clone)]
pub struct TimelinePanelState {
    pub position: Timestamp,
    pub duration: Timestamp,
    pub playback_state: ReplayState,
    pub speed: f64,
}

impl Default for TimelinePanelState {
    fn default() -> Self {
        Self {
            position: Timestamp::zero(),
            duration: Timestamp::zero(),
            playback_state: ReplayState::Idle,
            speed: 1.0,
        }
    }
}

type VoidCallback = Arc<dyn Fn() + Send + Sync>;
type SeekCallback = Arc<dyn Fn(f32) + Send + Sync>;
type SpeedCallback = Arc<dyn Fn(f64) + Send + Sync>;

struct TimelinePanelConfig {
    position: Timestamp,
    duration: Timestamp,
    playback_state: ReplayState,
    speed: f64,
    event_positions: Vec<f32>,
    on_step_back: Option<VoidCallback>,
    on_play_pause: Option<VoidCallback>,
    on_step_forward: Option<VoidCallback>,
    on_seek: Option<SeekCallback>,
    on_speed_change: Option<SpeedCallback>,
}

struct BuiltTimelinePanel {
    inner: Div,
}

impl BuiltTimelinePanel {
    const TRACK_WIDTH: f32 = 740.0;
    const EVENT_MARKER_WIDTH: f32 = 3.0;

    fn from_config(config: &TimelinePanelConfig) -> Self {
        let theme = ThemeState::get();

        let inner = div()
            .w_full()
            .py(2.0)
            .h(DebuggerTokens::TIMELINE_HEIGHT)
            .bg(theme.color(ColorToken::SurfaceElevated))
            .flex_col()
            .child(separator())
            .child(Self::controls(config))
            .child(
                div()
                    .w_full()
                    .flex_row()
                    .items_center()
                    .child(Self::timeline_track(config)),
            );

        BuiltTimelinePanel { inner }
    }

    fn controls(config: &TimelinePanelConfig) -> Div {
        let theme = ThemeState::get();
        let is_playing = config.playback_state == ReplayState::Playing;

        let speed_str = format!("{:.1}", config.speed);
        let speed_state =
            BlincContextState::get().use_state_keyed("timeline_speed", || speed_str.clone());
        if speed_state.get() != speed_str {
            speed_state.set(speed_str.clone());
        }

        let on_step_back = config.on_step_back.clone();
        let on_play_pause = config.on_play_pause.clone();
        let on_step_forward = config.on_step_forward.clone();
        let on_speed_change = config.on_speed_change.clone();

        div()
            .w_full()
            .h(44.0)
            .px(12.0)
            .py(8.0)
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .gap(2.0)
                    .child(
                        button("")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Icon)
                            .icon(icons::SKIP_BACK)
                            .on_click(move |_| {
                                if let Some(cb) = &on_step_back {
                                    cb();
                                }
                            }),
                    )
                    .child(
                        button("")
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Icon)
                            .icon(if is_playing {
                                icons::PAUSE
                            } else {
                                icons::PLAY
                            })
                            .on_click(move |_| {
                                if let Some(cb) = &on_play_pause {
                                    cb();
                                }
                            }),
                    )
                    .child(
                        button("")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Icon)
                            .icon(icons::SKIP_FORWARD)
                            .on_click(move |_| {
                                if let Some(cb) = &on_step_forward {
                                    cb();
                                }
                            }),
                    ),
            )
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .gap(4.0)
                    .child(
                        text(Self::format_time(config.position))
                            .size(12.0)
                            .color(theme.color(ColorToken::TextPrimary)),
                    )
                    .child(
                        text("/")
                            .size(12.0)
                            .color(theme.color(ColorToken::TextTertiary)),
                    )
                    .child(
                        text(Self::format_time(config.duration))
                            .size(12.0)
                            .color(theme.color(ColorToken::TextSecondary)),
                    ),
            )
            .child(
                select(&speed_state)
                    .size(SelectSize::Small)
                    .w(80.0)
                    .option("0.5", "0.5x")
                    .option("1.0", "1.0x")
                    .option("2.0", "2.0x")
                    .on_change(move |value| {
                        if let (Some(cb), Ok(speed)) = (&on_speed_change, value.parse::<f64>()) {
                            cb(speed);
                        }
                    }),
            )
    }

    fn timeline_track(config: &TimelinePanelConfig) -> Div {
        let position_norm = if config.duration.as_micros() > 0 {
            config.position.as_micros() as f32 / config.duration.as_micros() as f32
        } else {
            0.0
        };

        let position_state =
            BlincContextState::get().use_state_keyed("timeline_position", || position_norm);
        if (position_state.get() - position_norm).abs() > 0.0005 {
            position_state.set(position_norm);
        }

        let on_seek = config.on_seek.clone();

        div()
            .w_full()
            .padding_x_px(24.0)
            .py(4.0)
            .flex_col()
            .gap_px(8.0)
            .items_center()
            .justify_center()
            .child(Self::event_markers(&config.event_positions))
            .child(
                slider(&position_state)
                    .min(0.0)
                    .max(1.0)
                    .size(SliderSize::Small)
                    .w(Self::TRACK_WIDTH)
                    .on_change(move |value| {
                        if let Some(cb) = &on_seek {
                            cb(value.clamp(0.0, 1.0));
                        }
                    })
                    .build_final(),
            )
            .child(Self::time_labels(config.duration))
    }

    fn event_markers(positions: &[f32]) -> Div {
        let theme = ThemeState::get();
        let colors = [
            theme.color(ColorToken::Primary),
            theme.color(ColorToken::Info),
            theme.color(ColorToken::Secondary),
            theme.color(ColorToken::Accent),
            theme.color(ColorToken::Warning),
        ];

        let mut track = div().w(Self::TRACK_WIDTH).h(16.0).relative();
        for (idx, pos) in positions.iter().enumerate() {
            track = track.child(
                div()
                    .absolute()
                    .left(
                        (pos * Self::TRACK_WIDTH)
                            .clamp(0.0, Self::TRACK_WIDTH - Self::EVENT_MARKER_WIDTH),
                    )
                    .top(2.0)
                    .w(Self::EVENT_MARKER_WIDTH)
                    .h(12.0)
                    .rounded(1.5)
                    .bg(colors[idx % colors.len()]),
            );
        }
        track
    }

    fn time_labels(duration: Timestamp) -> Div {
        let theme = ThemeState::get();
        div()
            .w(Self::TRACK_WIDTH)
            .h(14.0)
            .flex_row()
            .justify_between()
            .child(
                text(Self::format_time(Timestamp::zero()))
                    .size(10.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            )
            .child(
                text(Self::format_time(duration))
                    .size(10.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            )
    }

    fn format_time(ts: Timestamp) -> String {
        let total_millis = ts.as_micros() / 1_000;
        let minutes = total_millis / 60_000;
        let seconds = (total_millis / 1_000) % 60;
        let millis = total_millis % 1_000;
        format!("{minutes}:{seconds:02}.{millis:03}")
    }
}

pub struct TimelinePanel {
    config: TimelinePanelConfig,
    built: OnceCell<BuiltTimelinePanel>,
}

impl TimelinePanel {
    const MAX_MARKERS: usize = 120;
    const MAX_TRACE_MARKERS: usize = 32;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        events: &[TimestampedEvent],
        trace_entries: &[TraceEntry],
        state: &TimelinePanelState,
        on_step_back: Option<VoidCallback>,
        on_play_pause: Option<VoidCallback>,
        on_step_forward: Option<VoidCallback>,
        on_seek: Option<SeekCallback>,
        on_speed_change: Option<SpeedCallback>,
    ) -> Self {
        Self {
            config: TimelinePanelConfig {
                position: state.position,
                duration: state.duration,
                playback_state: state.playback_state,
                speed: state.speed,
                event_positions: Self::sample_marker_positions(
                    events,
                    trace_entries,
                    state.duration,
                ),
                on_step_back,
                on_play_pause,
                on_step_forward,
                on_seek,
                on_speed_change,
            },
            built: OnceCell::new(),
        }
    }

    fn sample_marker_positions(
        events: &[TimestampedEvent],
        trace_entries: &[TraceEntry],
        duration: Timestamp,
    ) -> Vec<f32> {
        if (events.is_empty() && trace_entries.is_empty()) || duration.as_micros() == 0 {
            return Vec::new();
        }

        let mut positions = Self::sample_trace_marker_positions(trace_entries, duration);
        let event_budget = Self::MAX_MARKERS.saturating_sub(positions.len());
        positions.extend(Self::sample_event_positions(events, duration, event_budget));
        positions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        positions.dedup_by(|a, b| (*a - *b).abs() < 0.0005);
        positions
    }

    fn sample_event_positions(
        events: &[TimestampedEvent],
        duration: Timestamp,
        budget: usize,
    ) -> Vec<f32> {
        if budget == 0 || events.is_empty() {
            return Vec::new();
        }

        let positions = events
            .iter()
            .map(|event| {
                (event.timestamp.as_micros() as f32 / duration.as_micros() as f32).clamp(0.0, 1.0)
            })
            .collect::<Vec<_>>();
        Self::sample_sorted_positions(&positions, budget)
    }

    fn sample_trace_marker_positions(
        trace_entries: &[TraceEntry],
        duration: Timestamp,
    ) -> Vec<f32> {
        if trace_entries.is_empty() {
            return Vec::new();
        }

        let mut critical = Vec::new();
        let mut commands = Vec::new();
        let mut resolutions = Vec::new();

        for entry in trace_entries {
            let position =
                (entry.timestamp.as_micros() as f32 / duration.as_micros() as f32).clamp(0.0, 1.0);
            match &entry.kind {
                TraceEntryKind::Assertion(_) | TraceEntryKind::Artifact(_) => {
                    critical.push(position)
                }
                TraceEntryKind::Command(_) => commands.push(position),
                TraceEntryKind::LocatorResolution(_) => resolutions.push(position),
            }
        }

        let mut positions = Self::sample_sorted_positions(&critical, Self::MAX_TRACE_MARKERS);
        let remaining = Self::MAX_TRACE_MARKERS.saturating_sub(positions.len());
        positions.extend(Self::sample_sorted_positions(&commands, remaining));
        let remaining = Self::MAX_TRACE_MARKERS.saturating_sub(positions.len());
        positions.extend(Self::sample_sorted_positions(&resolutions, remaining));
        positions
    }

    fn sample_sorted_positions(positions: &[f32], budget: usize) -> Vec<f32> {
        if budget == 0 || positions.is_empty() {
            return Vec::new();
        }

        let mut sorted = positions.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if sorted.len() <= budget {
            return sorted;
        }
        if budget == 1 {
            return vec![*sorted.last().unwrap_or(&sorted[0])];
        }

        let last_index = sorted.len() - 1;
        let step = last_index as f32 / (budget - 1) as f32;
        (0..budget)
            .map(|idx| {
                let sample_index = ((idx as f32) * step).round() as usize;
                sorted[sample_index.min(last_index)]
            })
            .collect()
    }

    fn get_or_build(&self) -> &BuiltTimelinePanel {
        self.built
            .get_or_init(|| BuiltTimelinePanel::from_config(&self.config))
    }
}

impl ElementBuilder for TimelinePanel {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        self.get_or_build().inner.build(tree)
    }

    fn render_props(&self) -> RenderProps {
        self.get_or_build().inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.get_or_build().inner.children_builders()
    }

    fn event_handlers(&self) -> Option<&EventHandlers> {
        let handlers = self.get_or_build().inner.event_handlers();
        if handlers.is_empty() {
            None
        } else {
            Some(handlers)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TimelinePanel;
    use blinc_recorder::{
        CustomEvent, RecordedEvent, Timestamp, TimestampedEvent, TraceAssertionRecord, TraceEntry,
        TraceEntryKind,
    };

    #[test]
    fn timeline_sampling_preserves_assertion_markers_under_event_load() {
        let duration = Timestamp::from_micros(10_000);
        let events = (0..1_000)
            .map(|idx| TimestampedEvent {
                timestamp: Timestamp::from_micros(idx * 10),
                event: RecordedEvent::Custom(CustomEvent {
                    name: "tick".to_string(),
                    payload: None,
                }),
            })
            .collect::<Vec<_>>();
        let trace_entries = vec![TraceEntry {
            sequence: 1,
            timestamp: Timestamp::from_micros(5_555),
            kind: TraceEntryKind::Assertion(TraceAssertionRecord {
                code: "assert_text_contains".to_string(),
                passed: false,
                target: Some("status".to_string()),
                actual: Some("Ready".to_string()),
                expected: Some("Signed in".to_string()),
            }),
        }];

        let positions = TimelinePanel::sample_marker_positions(&events, &trace_entries, duration);

        assert!(
            positions
                .iter()
                .any(|position| (*position - 0.5555).abs() < 0.0001),
            "expected assertion marker to survive event sampling: {positions:?}"
        );
    }

    #[test]
    fn timeline_sampling_keeps_last_critical_marker() {
        let duration = Timestamp::from_micros(33_000);
        let trace_entries = (0..33)
            .map(|idx| TraceEntry {
                sequence: idx + 1,
                timestamp: Timestamp::from_micros((idx + 1) * 1_000),
                kind: TraceEntryKind::Assertion(TraceAssertionRecord {
                    code: format!("assert_{idx}"),
                    passed: idx != 32,
                    target: Some("status".to_string()),
                    actual: None,
                    expected: None,
                }),
            })
            .collect::<Vec<_>>();

        let positions = TimelinePanel::sample_marker_positions(&[], &trace_entries, duration);

        assert!(
            positions
                .iter()
                .any(|position| (*position - 1.0).abs() < 0.0001),
            "expected the final trace marker to survive sampling: {positions:?}"
        );
    }

    #[test]
    fn event_sampling_keeps_tail_markers_when_budget_is_nearly_exhausted() {
        let duration = Timestamp::from_micros(239_000);
        let events = (0..239)
            .map(|idx| TimestampedEvent {
                timestamp: Timestamp::from_micros((idx + 1) * 1_000),
                event: RecordedEvent::Custom(CustomEvent {
                    name: format!("tick-{idx}"),
                    payload: None,
                }),
            })
            .collect::<Vec<_>>();

        let positions = TimelinePanel::sample_marker_positions(&events, &[], duration);

        assert!(
            positions
                .iter()
                .any(|position| (*position - 1.0).abs() < 0.0001),
            "expected the final event marker to survive sampling: {positions:?}"
        );
    }

    #[test]
    fn format_time_includes_subsecond_precision() {
        assert_eq!(
            super::BuiltTimelinePanel::format_time(Timestamp::from_micros(61_234_000)),
            "1:01.234"
        );
    }
}
