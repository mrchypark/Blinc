//! Tree Panel - Element tree with selection

use std::cell::OnceCell;
use std::sync::Arc;

use blinc_cn::components::input::{input, InputSize};
use blinc_cn::components::separator::separator;
use blinc_cn::components::tree::{tree_view, TreeNodeConfig};
use blinc_layout::div::{Div, ElementBuilder};
use blinc_layout::element::RenderProps;
use blinc_layout::event_handler::EventHandlers;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_layout::widgets::text_input::text_input_data;
use blinc_recorder::TreeSnapshot;
use blinc_theme::{ColorToken, ThemeState};

use crate::theme::DebuggerTokens;

/// State for the tree panel
#[derive(Default)]
pub struct TreePanelState;

type SelectCallback = Arc<dyn Fn(String) + Send + Sync>;

struct TreePanelConfig {
    snapshot: Option<TreeSnapshot>,
    selected_id: Option<String>,
    on_select: Option<SelectCallback>,
}

struct BuiltTreePanel {
    inner: Div,
}

impl BuiltTreePanel {
    fn from_config(config: &TreePanelConfig) -> Self {
        let theme = ThemeState::get();

        let inner = div()
            .w(DebuggerTokens::TREE_PANEL_WIDTH)
            .h_full()
            .bg(theme.color(ColorToken::SurfaceElevated))
            .flex_col()
            .child(Self::header(config.snapshot.as_ref()))
            .child(separator())
            .child(Self::search_bar())
            .child(Self::tree_content(config))
            .child(separator());

        BuiltTreePanel { inner }
    }

    fn header(snapshot: Option<&TreeSnapshot>) -> Div {
        let theme = ThemeState::get();
        let count = snapshot.map(|s| s.elements.len()).unwrap_or(0);
        div()
            .h(44.0)
            .px(12.0)
            .py(2.0)
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                text("Element Tree")
                    .size(13.0)
                    .color(theme.color(ColorToken::TextPrimary))
                    .weight(blinc_layout::div::FontWeight::SemiBold),
            )
            .child(
                text(format!("{count}"))
                    .size(11.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            )
    }

    fn search_bar() -> Div {
        let search_data = text_input_data();
        div().px(8.0).py(6.0).child(
            input(&search_data)
                .placeholder("Search...")
                .size(InputSize::Small),
        )
    }

    fn tree_content(config: &TreePanelConfig) -> Scroll {
        let content = if let Some(snapshot) = config.snapshot.as_ref() {
            Self::render_tree(
                snapshot,
                config.selected_id.as_ref(),
                config.on_select.clone(),
            )
        } else {
            Self::render_empty_state()
        };

        scroll().flex_grow().child(content)
    }

    fn render_tree(
        snapshot: &TreeSnapshot,
        selected_id: Option<&String>,
        on_select: Option<SelectCallback>,
    ) -> Div {
        let mut tree = tree_view().indent(12.0);

        if let Some(selected) = selected_id {
            tree = tree.selected(selected.clone());
        }

        if let Some(root_id) = snapshot.root_id.as_deref() {
            if let Some(node) = Self::build_node(snapshot, root_id) {
                tree = tree.add_node(node.expanded());
            }
        } else {
            let mut roots: Vec<_> = snapshot
                .elements
                .values()
                .filter(|e| e.parent.is_none())
                .map(|e| e.id.as_str())
                .collect();
            roots.sort_unstable();
            for root in roots {
                if let Some(node) = Self::build_node(snapshot, root) {
                    tree = tree.add_node(node.expanded());
                }
            }
        }

        if let Some(on_select_cb) = on_select {
            tree = tree.on_select(move |key| on_select_cb(key.to_string()));
        }

        div().child(tree)
    }

    fn build_node(snapshot: &TreeSnapshot, id: &str) -> Option<TreeNodeConfig> {
        let element = snapshot.elements.get(id)?;
        let mut node = TreeNodeConfig::new(
            id.to_string(),
            format!("{} · {}", element.element_type, element.id),
        );

        if !element.children.is_empty() {
            node = node.expanded();
        }

        for child in &element.children {
            if let Some(child_node) = Self::build_node(snapshot, child) {
                node.children.push(child_node);
            }
        }

        Some(node)
    }

    fn render_empty_state() -> Div {
        let theme = ThemeState::get();
        div()
            .w_full()
            .h_full()
            .items_center()
            .justify_center()
            .child(
                text("No recording loaded")
                    .size(13.0)
                    .color(theme.color(ColorToken::TextTertiary)),
            )
    }
}

pub struct TreePanel {
    config: TreePanelConfig,
    built: OnceCell<BuiltTreePanel>,
}

impl TreePanel {
    pub fn new(
        snapshot: Option<&TreeSnapshot>,
        selected_id: Option<&String>,
        _state: &TreePanelState,
        on_select: Option<SelectCallback>,
    ) -> Self {
        Self {
            config: TreePanelConfig {
                snapshot: snapshot.cloned(),
                selected_id: selected_id.cloned(),
                on_select,
            },
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &BuiltTreePanel {
        self.built
            .get_or_init(|| BuiltTreePanel::from_config(&self.config))
    }
}

impl ElementBuilder for TreePanel {
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
