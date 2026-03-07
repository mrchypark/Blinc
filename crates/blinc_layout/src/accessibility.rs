use std::sync::Arc;

use blinc_platform::{
    AccessibilityAction, AccessibilityBounds, AccessibilityNode, AccessibilityRole,
    AccessibilityTreeSnapshot,
};

use crate::renderer::{ElementType, RenderTree};
use crate::tree::LayoutNodeId;

#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityMetadata {
    pub role: AccessibilityRole,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub focusable: bool,
    pub focused: bool,
    pub disabled: bool,
    pub actions: Vec<AccessibilityAction>,
}

impl AccessibilityMetadata {
    pub fn new(role: AccessibilityRole) -> Self {
        Self {
            role,
            name: None,
            description: None,
            value: None,
            focusable: false,
            focused: false,
            disabled: false,
            actions: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_value(mut self, value: Option<String>) -> Self {
        self.value = value;
        self
    }

    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_actions(mut self, actions: Vec<AccessibilityAction>) -> Self {
        self.actions = actions;
        self
    }
}

pub type AccessibilityMetadataProvider = Arc<dyn Fn() -> AccessibilityMetadata + Send + Sync>;

pub fn export_accessibility_snapshot(tree: &RenderTree) -> Option<AccessibilityTreeSnapshot> {
    let root = tree.root()?;
    Some(export_snapshot(tree, root))
}

fn export_snapshot(tree: &RenderTree, root: LayoutNodeId) -> AccessibilityTreeSnapshot {
    let mut nodes = Vec::new();
    let children = collect_nodes(tree, root, &mut nodes, true);
    let root_id = root.to_raw();

    if !nodes.iter().any(|node| node.id == root_id) {
        let bounds = tree.get_absolute_bounds(root).map(|bounds| {
            AccessibilityBounds::new(bounds.x, bounds.y, bounds.width, bounds.height)
        });
        nodes.insert(
            0,
            AccessibilityNode {
                id: root_id,
                role: AccessibilityRole::Group,
                name: None,
                description: None,
                value: None,
                bounds,
                focusable: false,
                focused: false,
                disabled: false,
                actions: Vec::new(),
                children,
            },
        );
    }

    AccessibilityTreeSnapshot::new(root_id, nodes)
}

fn collect_nodes(
    tree: &RenderTree,
    node_id: LayoutNodeId,
    nodes: &mut Vec<AccessibilityNode>,
    is_root: bool,
) -> Vec<u64> {
    let mut exported_children = Vec::new();
    for child in tree.layout_tree.children(node_id) {
        exported_children.extend(collect_nodes(tree, child, nodes, false));
    }

    let metadata = tree.layout_tree.accessibility_metadata(node_id);
    let accessibility_id = node_id.to_raw();
    let bounds = tree
        .get_absolute_bounds(node_id)
        .map(|bounds| AccessibilityBounds::new(bounds.x, bounds.y, bounds.width, bounds.height));

    let Some(metadata) = metadata else {
        if !is_root && !exported_children.is_empty() {
            nodes.push(AccessibilityNode {
                id: accessibility_id,
                role: AccessibilityRole::Group,
                name: None,
                description: None,
                value: None,
                bounds,
                focusable: false,
                focused: false,
                disabled: false,
                actions: Vec::new(),
                children: exported_children,
            });
            return vec![accessibility_id];
        }
        return exported_children;
    };

    let name = match metadata.role {
        AccessibilityRole::Button | AccessibilityRole::Checkbox | AccessibilityRole::Label => {
            metadata
                .name
                .clone()
                .or_else(|| infer_descendant_text(tree, node_id))
        }
        _ => metadata.name.clone(),
    };

    nodes.push(AccessibilityNode {
        id: accessibility_id,
        role: metadata.role,
        name,
        description: metadata.description,
        value: metadata.value,
        bounds,
        focusable: metadata.focusable,
        focused: metadata.focused,
        disabled: metadata.disabled,
        actions: metadata.actions,
        children: exported_children,
    });

    vec![accessibility_id]
}

fn infer_descendant_text(tree: &RenderTree, node_id: LayoutNodeId) -> Option<String> {
    let mut parts = Vec::new();
    collect_descendant_text(tree, node_id, &mut parts, true);
    let name = parts.join(" ");
    let trimmed = name.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn collect_descendant_text(
    tree: &RenderTree,
    node_id: LayoutNodeId,
    parts: &mut Vec<String>,
    is_root: bool,
) {
    if !is_root && tree.layout_tree.accessibility_metadata(node_id).is_some() {
        return;
    }

    if let Some(render_node) = tree.get_render_node(node_id) {
        match &render_node.element_type {
            ElementType::Text(text) => {
                let trimmed = text.content.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
            ElementType::StyledText(text) => {
                let trimmed = text.content.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    for child in tree.layout_tree.children(node_id) {
        collect_descendant_text(tree, child, parts, false);
    }
}

pub fn focus_order(snapshot: &AccessibilityTreeSnapshot) -> Vec<u64> {
    snapshot
        .nodes
        .iter()
        .filter(|node| node.focusable && !node.disabled)
        .map(|node| node.id)
        .collect()
}
