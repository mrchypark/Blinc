//! Layout tree management

use slotmap::{new_key_type, SlotMap};
use taffy::prelude::*;

new_key_type! {
    pub struct LayoutNodeId;
}

/// Maps between Blinc node IDs and Taffy node IDs
pub struct LayoutTree {
    taffy: TaffyTree,
    node_map: SlotMap<LayoutNodeId, NodeId>,
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: SlotMap::with_key(),
        }
    }

    /// Create a new layout node with the given style
    pub fn create_node(&mut self, style: Style) -> LayoutNodeId {
        let taffy_node = self.taffy.new_leaf(style).unwrap();
        self.node_map.insert(taffy_node)
    }

    /// Set the style for a node
    pub fn set_style(&mut self, id: LayoutNodeId, style: Style) {
        if let Some(&taffy_node) = self.node_map.get(id) {
            let _ = self.taffy.set_style(taffy_node, style);
        }
    }

    /// Add a child to a parent node
    pub fn add_child(&mut self, parent: LayoutNodeId, child: LayoutNodeId) {
        if let (Some(&parent_node), Some(&child_node)) =
            (self.node_map.get(parent), self.node_map.get(child))
        {
            let _ = self.taffy.add_child(parent_node, child_node);
        }
    }

    /// Compute layout for a tree rooted at the given node
    pub fn compute_layout(&mut self, root: LayoutNodeId, available_space: Size<AvailableSpace>) {
        if let Some(&taffy_node) = self.node_map.get(root) {
            let _ = self.taffy.compute_layout(taffy_node, available_space);
        }
    }

    /// Get the computed layout for a node
    pub fn get_layout(&self, id: LayoutNodeId) -> Option<&Layout> {
        self.node_map
            .get(id)
            .and_then(|&taffy_node| self.taffy.layout(taffy_node).ok())
    }

    /// Remove a node
    pub fn remove_node(&mut self, id: LayoutNodeId) {
        if let Some(taffy_node) = self.node_map.remove(id) {
            let _ = self.taffy.remove(taffy_node);
        }
    }
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}
