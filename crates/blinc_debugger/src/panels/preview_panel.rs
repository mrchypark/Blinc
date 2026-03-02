//! Preview Panel - Snapshot preview and overlays

use std::cell::OnceCell;
use std::sync::Arc;

use blinc_cn::components::select::{select, SelectSize};
use blinc_cn::components::separator::separator;
use blinc_cn::components::switch::{switch, SwitchSize};
use blinc_core::context_state::BlincContextState;
use blinc_core::Color;
use blinc_layout::div::{Div, ElementBuilder, FontWeight};
use blinc_layout::element::RenderProps;
use blinc_layout::event_handler::EventHandlers;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_recorder::TreeSnapshot;
use blinc_theme::{ColorToken, ThemeState};

#[derive(Clone)]
pub struct PreviewConfig {
    pub show_bounds: bool,
    pub show_cursor: bool,
    pub zoom: f32,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            show_bounds: false,
            show_cursor: true,
            zoom: 1.0,
        }
    }
}

type BoolCallback = Arc<dyn Fn(bool) + Send + Sync>;
type ZoomCallback = Arc<dyn Fn(f32) + Send + Sync>;

struct PreviewPanelConfig {
    has_snapshot: bool,
    show_bounds: bool,
    show_cursor: bool,
    zoom: f32,
    cursor_position: Option<(f32, f32)>,
    window_size: (u32, u32),
    element_count: usize,
    on_show_bounds_change: Option<BoolCallback>,
    on_show_cursor_change: Option<BoolCallback>,
    on_zoom_change: Option<ZoomCallback>,
}

struct BuiltPreviewPanel {
    inner: Div,
}

impl BuiltPreviewPanel {
    fn from_config(config: &PreviewPanelConfig) -> Self {
        let theme = ThemeState::get();

        let inner = div()
            .flex_grow()
            .h_full()
            .bg(theme.color(ColorToken::Background))
            .flex_col()
            .child(Self::toolbar(config))
            .child(separator())
            .child(Self::preview_area(config));

        BuiltPreviewPanel { inner }
    }

    fn toolbar(config: &PreviewPanelConfig) -> Div {
        let theme = ThemeState::get();
        let ctx = BlincContextState::get();

        let bounds_state = ctx.use_state_keyed("preview_bounds", || config.show_bounds);
        if bounds_state.get() != config.show_bounds {
            bounds_state.set(config.show_bounds);
        }

        let cursor_state = ctx.use_state_keyed("preview_cursor", || config.show_cursor);
        if cursor_state.get() != config.show_cursor {
            cursor_state.set(config.show_cursor);
        }

        let zoom_str = format!("{}", (config.zoom * 100.0) as i32);
        let zoom_state = ctx.use_state_keyed("preview_zoom", || zoom_str.clone());
        if zoom_state.get() != zoom_str {
            zoom_state.set(zoom_str.clone());
        }

        let on_show_bounds_change = config.on_show_bounds_change.clone();
        let on_show_cursor_change = config.on_show_cursor_change.clone();
        let on_zoom_change = config.on_zoom_change.clone();

        div()
            .h(44.0)
            .py(2.0)
            .px(12.0)
            .bg(theme.color(ColorToken::SurfaceElevated))
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                text(format!(
                    "Preview · {}x{} · {} elements",
                    config.window_size.0, config.window_size.1, config.element_count
                ))
                .size(13.0)
                .color(theme.color(ColorToken::TextPrimary))
                .weight(FontWeight::SemiBold),
            )
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .gap(12.0)
                    .child(
                        switch(&bounds_state)
                            .size(SwitchSize::Small)
                            .label("Bounds")
                            .on_change(move |value| {
                                if let Some(cb) = &on_show_bounds_change {
                                    cb(value);
                                }
                            }),
                    )
                    .child(
                        switch(&cursor_state)
                            .size(SwitchSize::Small)
                            .label("Cursor")
                            .on_change(move |value| {
                                if let Some(cb) = &on_show_cursor_change {
                                    cb(value);
                                }
                            }),
                    )
                    .child(
                        select(&zoom_state)
                            .size(SelectSize::Small)
                            .w(80.0)
                            .option("50", "50%")
                            .option("75", "75%")
                            .option("100", "100%")
                            .option("150", "150%")
                            .option("200", "200%")
                            .on_change(move |value| {
                                if let (Some(cb), Ok(percent)) =
                                    (&on_zoom_change, value.parse::<f32>())
                                {
                                    cb((percent / 100.0).clamp(0.25, 3.0));
                                }
                            }),
                    ),
            )
    }

    fn preview_area(config: &PreviewPanelConfig) -> Div {
        let content = if config.has_snapshot {
            Self::render_preview(config)
        } else {
            Self::render_empty_state()
        };

        div()
            .flex_grow()
            .overflow_clip()
            .items_center()
            .justify_center()
            .p(16.0)
            .child(content)
    }

    fn render_preview(config: &PreviewPanelConfig) -> Div {
        let theme = ThemeState::get();
        let width = config.window_size.0.max(1) as f32 * config.zoom;
        let height = config.window_size.1.max(1) as f32 * config.zoom;

        let mut preview = div()
            .w(width)
            .h(height)
            .bg(theme.color(ColorToken::Surface))
            .rounded(8.0)
            .border(1.0, theme.color(ColorToken::Border))
            .relative()
            .child(
                div().absolute().left(12.0).top(10.0).child(
                    text("Snapshot")
                        .size(14.0)
                        .color(theme.color(ColorToken::TextTertiary)),
                ),
            );

        if config.show_bounds {
            preview = preview.child(
                div()
                    .absolute()
                    .left(8.0)
                    .top(36.0)
                    .right(8.0)
                    .bottom(8.0)
                    .border(1.0, theme.color(ColorToken::Info).with_alpha(0.5))
                    .rounded(6.0),
            );
        }

        if config.show_cursor {
            if let Some((x, y)) = config.cursor_position {
                preview = preview.child(Self::render_cursor(x, y, config.zoom));
            }
        }

        preview
    }

    fn render_cursor(x: f32, y: f32, zoom: f32) -> Div {
        let theme = ThemeState::get();
        div()
            .absolute()
            .left(x * zoom)
            .top(y * zoom)
            .w(10.0)
            .h(10.0)
            .rounded_full()
            .bg(theme.color(ColorToken::Primary))
            .border(2.0, Color::WHITE)
    }

    fn render_empty_state() -> Div {
        let theme = ThemeState::get();
        div()
            .w(400.0)
            .flex_col()
            .items_center()
            .gap(8.0)
            .child(
                text("No Preview Available")
                    .size(16.0)
                    .color(theme.color(ColorToken::TextPrimary)),
            )
            .child(
                text("Load a recording or connect to a running app")
                    .size(13.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            )
    }
}

pub struct PreviewPanel {
    config: PreviewPanelConfig,
    built: OnceCell<BuiltPreviewPanel>,
}

impl PreviewPanel {
    pub fn new(
        snapshot: Option<&TreeSnapshot>,
        config: &PreviewConfig,
        cursor_position: Option<(f32, f32)>,
        on_show_bounds_change: Option<BoolCallback>,
        on_show_cursor_change: Option<BoolCallback>,
        on_zoom_change: Option<ZoomCallback>,
    ) -> Self {
        Self {
            config: PreviewPanelConfig {
                has_snapshot: snapshot.is_some(),
                show_bounds: config.show_bounds,
                show_cursor: config.show_cursor,
                zoom: config.zoom,
                cursor_position,
                window_size: snapshot.map(|s| s.window_size).unwrap_or((800, 600)),
                element_count: snapshot.map(|s| s.elements.len()).unwrap_or(0),
                on_show_bounds_change,
                on_show_cursor_change,
                on_zoom_change,
            },
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &BuiltPreviewPanel {
        self.built
            .get_or_init(|| BuiltPreviewPanel::from_config(&self.config))
    }
}

impl ElementBuilder for PreviewPanel {
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
