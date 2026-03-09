/// Command Panel - recorded automation commands
use std::cell::OnceCell;
use std::sync::Arc;

use blinc_cn::components::separator::separator;
use blinc_layout::div::{Div, ElementBuilder, FontWeight};
use blinc_layout::element::RenderProps;
use blinc_layout::event_handler::EventHandlers;
use blinc_layout::prelude::*;
use blinc_layout::selector::ScrollRef;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_theme::{ColorToken, ThemeState};

struct CommandPanelConfig {
    lines: Arc<[String]>,
    scroll_ref: ScrollRef,
}

struct BuiltCommandPanel {
    inner: Div,
}

impl BuiltCommandPanel {
    fn from_config(config: &CommandPanelConfig) -> Self {
        let theme = ThemeState::get();
        let inner = div()
            .w_full()
            .h(160.0)
            .bg(theme.color(ColorToken::SurfaceElevated))
            .flex_col()
            .child(Self::header())
            .child(separator())
            .child(Self::content(&config.lines, &config.scroll_ref));
        Self { inner }
    }

    fn header() -> Div {
        let theme = ThemeState::get();
        div().h(40.0).px(12.0).items_center().child(
            text("Commands")
                .size(12.0)
                .weight(FontWeight::SemiBold)
                .color(theme.color(ColorToken::TextPrimary)),
        )
    }

    fn content(lines: &[String], scroll_ref: &ScrollRef) -> Scroll {
        let theme = ThemeState::get();
        let mut body = div().flex_col().gap(4.0);
        if lines.is_empty() {
            body = body.child(
                text("No commands recorded")
                    .size(12.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            );
        } else {
            for line in lines {
                body = body.child(
                    text(line)
                        .size(12.0)
                        .color(theme.color(ColorToken::TextSecondary)),
                );
            }
        }
        scroll().bind(scroll_ref).flex_grow().p(8.0).child(body)
    }
}

pub struct CommandPanel {
    config: CommandPanelConfig,
    built: OnceCell<BuiltCommandPanel>,
}

impl CommandPanel {
    pub fn new(lines: Arc<[String]>) -> Self {
        Self::with_scroll_ref(lines, ScrollRef::new())
    }

    pub fn with_scroll_ref(lines: Arc<[String]>, scroll_ref: ScrollRef) -> Self {
        Self {
            config: CommandPanelConfig { lines, scroll_ref },
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &BuiltCommandPanel {
        self.built
            .get_or_init(|| BuiltCommandPanel::from_config(&self.config))
    }
}

impl ElementBuilder for CommandPanel {
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
