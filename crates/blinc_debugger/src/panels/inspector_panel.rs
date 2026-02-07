//! Inspector Panel - Selected element properties

use std::cell::OnceCell;

use blinc_cn::components::separator::separator;
use blinc_layout::div::{Div, ElementBuilder, FontWeight};
use blinc_layout::element::RenderProps;
use blinc_layout::event_handler::EventHandlers;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_recorder::ElementSnapshot;
use blinc_theme::{ColorToken, ThemeState};

use crate::theme::DebuggerTokens;

struct InspectorPanelConfig {
    element_id: Option<String>,
    element_bounds: Option<blinc_recorder::capture::Rect>,
    is_visible: bool,
    is_focused: bool,
}

struct BuiltInspectorPanel {
    inner: Div,
}

impl BuiltInspectorPanel {
    fn from_config(config: &InspectorPanelConfig) -> Self {
        let theme = ThemeState::get();

        let inner = div()
            .w(DebuggerTokens::INSPECTOR_WIDTH)
            .h_full()
            .bg(theme.color(ColorToken::SurfaceElevated))
            .flex_col()
            .child(Self::header())
            .child(separator())
            .child(Self::content(config))
            .child(separator());

        BuiltInspectorPanel { inner }
    }

    fn header() -> Div {
        let theme = ThemeState::get();
        div()
            .h(44.0)
            .px(12.0)
            .py(2.0)
            .flex_row()
            .items_center()
            .child(
                text("Inspector")
                    .size(13.0)
                    .color(theme.color(ColorToken::TextPrimary))
                    .weight(FontWeight::SemiBold),
            )
    }

    fn content(config: &InspectorPanelConfig) -> Scroll {
        let inner = if config.element_id.is_some() {
            Self::render_element_info(config)
        } else {
            Self::render_empty_state()
        };

        scroll().flex_grow().vertical().p(8.0).child(inner)
    }

    fn render_element_info(config: &InspectorPanelConfig) -> Div {
        let element_id = config.element_id.as_deref().unwrap_or("unknown");

        let mut container = div().flex_col().gap(12.0).child(Self::section(
            "Element",
            vec![("ID", element_id), ("Type", "div")],
        ));

        if let Some(bounds) = &config.element_bounds {
            container = container.child(Self::bounds_section(bounds));
        }

        container.child(Self::section(
            "State",
            vec![
                ("Visible", if config.is_visible { "Yes" } else { "No" }),
                ("Focused", if config.is_focused { "Yes" } else { "No" }),
            ],
        ))
    }

    fn section(title: &str, properties: Vec<(&str, &str)>) -> Div {
        let theme = ThemeState::get();
        let mut props = div().flex_col().gap(2.0);

        for (key, value) in properties {
            props = props.child(Self::property_row(key, value));
        }

        div()
            .flex_col()
            .gap(4.0)
            .child(
                text(title)
                    .size(11.0)
                    .color(theme.color(ColorToken::TextTertiary))
                    .weight(FontWeight::SemiBold),
            )
            .child(props)
    }

    fn bounds_section(bounds: &blinc_recorder::capture::Rect) -> Div {
        let theme = ThemeState::get();
        div()
            .flex_col()
            .gap(4.0)
            .child(
                text("Bounds")
                    .size(11.0)
                    .color(theme.color(ColorToken::TextTertiary))
                    .weight(FontWeight::SemiBold),
            )
            .child(
                div()
                    .flex_col()
                    .gap(2.0)
                    .child(Self::property_row_value("X", bounds.x))
                    .child(Self::property_row_value("Y", bounds.y))
                    .child(Self::property_row_value("Width", bounds.width))
                    .child(Self::property_row_value("Height", bounds.height)),
            )
    }

    fn property_row(key: &str, value: &str) -> Div {
        let theme = ThemeState::get();
        div()
            .flex_row()
            .justify_between()
            .child(
                text(key)
                    .size(12.0)
                    .color(theme.color(ColorToken::TextSecondary)),
            )
            .child(
                text(value)
                    .size(12.0)
                    .color(theme.color(ColorToken::TextPrimary)),
            )
    }

    fn property_row_value(key: &str, value: f32) -> Div {
        let theme = ThemeState::get();
        div()
            .flex_row()
            .justify_between()
            .child(
                text(key)
                    .size(12.0)
                    .color(theme.color(ColorToken::TextSecondary)),
            )
            .child(
                text(format!("{:.1}", value))
                    .size(12.0)
                    .color(theme.color(ColorToken::Primary)),
            )
    }

    fn render_empty_state() -> Div {
        let theme = ThemeState::get();
        div()
            .w_full()
            .h_full()
            .items_center()
            .justify_center()
            .child(
                text("Select an element")
                    .size(13.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            )
    }
}

pub struct InspectorPanel {
    config: InspectorPanelConfig,
    built: OnceCell<BuiltInspectorPanel>,
}

impl InspectorPanel {
    pub fn new(selected: Option<&ElementSnapshot>) -> Self {
        Self {
            config: InspectorPanelConfig {
                element_id: selected.map(|e| e.id.clone()),
                element_bounds: selected.map(|e| e.bounds.clone()),
                is_visible: selected.map(|e| e.is_visible).unwrap_or(false),
                is_focused: selected.map(|e| e.is_focused).unwrap_or(false),
            },
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &BuiltInspectorPanel {
        self.built
            .get_or_init(|| BuiltInspectorPanel::from_config(&self.config))
    }
}

impl ElementBuilder for InspectorPanel {
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
