//! Evidence Panel - assertion failures and artifacts

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

struct EvidencePanelConfig {
    lines: Arc<[String]>,
    scroll_ref: ScrollRef,
}

struct BuiltEvidencePanel {
    inner: Div,
}

impl BuiltEvidencePanel {
    fn locator_line_is_error(line: &str) -> bool {
        line.contains("[no_match]")
            || line.contains("[unresolved]")
            || line.contains("[ambiguous_match]")
            || line.contains("[nth_out_of_range]")
            || line.contains("[empty_tree]")
            || line.contains("[within_scope_not_found]")
    }

    fn line_color(theme: &ThemeState, line: &str) -> blinc_core::Color {
        if line.starts_with("FAIL ")
            || (line.starts_with("locator ") && Self::locator_line_is_error(line))
        {
            theme.color(ColorToken::Error)
        } else if line.starts_with("PASS ") {
            theme.color(ColorToken::Success)
        } else if line.starts_with("artifact ") || line.starts_with("locator ") {
            theme.color(ColorToken::Info)
        } else {
            theme.color(ColorToken::TextSecondary)
        }
    }

    fn from_config(config: &EvidencePanelConfig) -> Self {
        let theme = ThemeState::get();
        let inner = div()
            .w(280.0)
            .h_full()
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
            text("Evidence")
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
                text("No evidence")
                    .size(12.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            );
        } else {
            for line in lines {
                let color = Self::line_color(&theme, line);
                body = body.child(text(line).size(12.0).color(color));
            }
        }
        scroll().bind(scroll_ref).flex_grow().p(8.0).child(body)
    }
}

pub struct EvidencePanel {
    config: EvidencePanelConfig,
    built: OnceCell<BuiltEvidencePanel>,
}

impl EvidencePanel {
    pub fn new(lines: Arc<[String]>, scroll_ref: ScrollRef) -> Self {
        Self {
            config: EvidencePanelConfig { lines, scroll_ref },
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &BuiltEvidencePanel {
        self.built
            .get_or_init(|| BuiltEvidencePanel::from_config(&self.config))
    }
}

impl ElementBuilder for EvidencePanel {
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
    use super::BuiltEvidencePanel;
    use blinc_theme::{ColorToken, ThemeState};

    #[test]
    fn locator_failures_use_error_color() {
        ThemeState::init_default();
        let theme = ThemeState::get();
        assert_eq!(
            BuiltEvidencePanel::line_color(&theme, "locator role=Button [no_match]"),
            theme.color(ColorToken::Error)
        );
        assert_eq!(
            BuiltEvidencePanel::line_color(&theme, "locator role=Button [nth_out_of_range]"),
            theme.color(ColorToken::Error)
        );
        assert_eq!(
            BuiltEvidencePanel::line_color(&theme, "locator role=Button -> submit [matched]"),
            theme.color(ColorToken::Info)
        );
        assert_eq!(
            BuiltEvidencePanel::line_color(&theme, "locator role=Button [candidate]"),
            theme.color(ColorToken::Info)
        );
    }
}
