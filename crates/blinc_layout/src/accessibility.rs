use std::sync::Arc;

use blinc_platform::{
    AccessibilityAction, AccessibilityBounds, AccessibilityNode, AccessibilityRole,
    AccessibilityTreeSnapshot,
};

use crate::renderer::RenderTree;
use crate::tree::{LayoutNodeId, LayoutTree};

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
    Some(export_snapshot_from_layout(&tree.layout_tree, root))
}

fn export_snapshot_from_layout(tree: &LayoutTree, root: LayoutNodeId) -> AccessibilityTreeSnapshot {
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
    tree: &LayoutTree,
    node_id: LayoutNodeId,
    nodes: &mut Vec<AccessibilityNode>,
    is_root: bool,
) -> Vec<u64> {
    let mut exported_children = Vec::new();
    for child in tree.children(node_id) {
        exported_children.extend(collect_nodes(tree, child, nodes, false));
    }

    let Some(metadata) = tree.accessibility_metadata(node_id) else {
        return exported_children;
    };

    let bounds = tree
        .get_absolute_bounds(node_id)
        .map(|bounds| AccessibilityBounds::new(bounds.x, bounds.y, bounds.width, bounds.height));
    let accessibility_id = node_id.to_raw();

    nodes.push(AccessibilityNode {
        id: accessibility_id,
        role: metadata.role,
        name: metadata.name,
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

pub fn focus_order(snapshot: &AccessibilityTreeSnapshot) -> Vec<u64> {
    snapshot
        .nodes
        .iter()
        .filter(|node| node.focusable && !node.disabled)
        .map(|node| node.id)
        .collect()
}
