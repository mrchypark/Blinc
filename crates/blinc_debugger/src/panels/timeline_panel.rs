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
use blinc_recorder::{Timestamp, TimestampedEvent};
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
                text("0:00")
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
        let total_secs = ts.as_micros() / 1_000_000;
        format!("{}:{:02}", total_secs / 60, total_secs % 60)
    }
}

pub struct TimelinePanel {
    config: TimelinePanelConfig,
    built: OnceCell<BuiltTimelinePanel>,
}

impl TimelinePanel {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        events: &[TimestampedEvent],
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
                event_positions: Self::sample_event_positions(events, state.duration),
                on_step_back,
                on_play_pause,
                on_step_forward,
                on_seek,
                on_speed_change,
            },
            built: OnceCell::new(),
        }
    }

    fn sample_event_positions(events: &[TimestampedEvent], duration: Timestamp) -> Vec<f32> {
        if events.is_empty() || duration.as_micros() == 0 {
            return Vec::new();
        }

        let max_markers = 120usize;
        let stride = (events.len() / max_markers).max(1);
        events
            .iter()
            .step_by(stride)
            .map(|e| (e.timestamp.as_micros() as f32 / duration.as_micros() as f32).clamp(0.0, 1.0))
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
