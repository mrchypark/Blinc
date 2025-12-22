//! RenderTree bridge connecting layout to rendering
//!
//! This module provides the bridge between Taffy layout computation
//! and the DrawContext rendering API.

use std::collections::HashMap;

use blinc_core::{DrawContext, Rect, Transform};
use taffy::prelude::*;

use crate::div::ElementBuilder;
use crate::element::{ElementBounds, RenderLayer, RenderProps};
use crate::text::Text;
use crate::tree::{LayoutNodeId, LayoutTree};

/// Stores an element's type for rendering
#[derive(Clone)]
pub enum ElementType {
    /// A div/container element
    Div,
    /// A text element with content
    Text(TextData),
}

/// Text data for rendering
#[derive(Clone)]
pub struct TextData {
    pub content: String,
    pub font_size: f32,
    pub color: [f32; 4],
}

/// Node data for rendering
#[derive(Clone)]
pub struct RenderNode {
    /// Render properties
    pub props: RenderProps,
    /// Element type
    pub element_type: ElementType,
}

/// RenderTree - bridges layout computation and rendering
pub struct RenderTree {
    /// The underlying layout tree
    pub layout_tree: LayoutTree,
    /// Render data for each node
    render_nodes: HashMap<LayoutNodeId, RenderNode>,
    /// Root node ID
    root: Option<LayoutNodeId>,
}

impl Default for RenderTree {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderTree {
    /// Create a new empty render tree
    pub fn new() -> Self {
        Self {
            layout_tree: LayoutTree::new(),
            render_nodes: HashMap::new(),
            root: None,
        }
    }

    /// Build a render tree from an element builder
    pub fn from_element<E: ElementBuilder>(element: &E) -> Self {
        let mut tree = Self::new();
        tree.root = Some(tree.build_element(element));
        tree
    }

    /// Recursively build elements into the tree
    fn build_element<E: ElementBuilder>(&mut self, element: &E) -> LayoutNodeId {
        let node_id = element.build(&mut self.layout_tree);
        let mut props = element.render_props();
        props.node_id = Some(node_id);

        // Determine element type
        let element_type = if let Some(text) = Self::try_as_text(element) {
            ElementType::Text(TextData {
                content: text.content().to_string(),
                font_size: text.font_size(),
                color: [
                    text.text_color().r,
                    text.text_color().g,
                    text.text_color().b,
                    text.text_color().a,
                ],
            })
        } else {
            ElementType::Div
        };

        self.render_nodes.insert(
            node_id,
            RenderNode {
                props,
                element_type,
            },
        );

        // Build children
        for child in element.children_builders() {
            self.build_element_boxed(child.as_ref());
        }

        node_id
    }

    /// Build from a boxed element builder
    fn build_element_boxed(&mut self, element: &dyn ElementBuilder) -> LayoutNodeId {
        let node_id = element.build(&mut self.layout_tree);
        let mut props = element.render_props();
        props.node_id = Some(node_id);

        // For boxed elements, we can't easily determine if it's Text
        // This would need trait-based type detection in production
        let element_type = ElementType::Div;

        self.render_nodes.insert(
            node_id,
            RenderNode {
                props,
                element_type,
            },
        );

        node_id
    }

    /// Try to cast element as Text (type detection helper)
    fn try_as_text<E: ElementBuilder>(_element: &E) -> Option<&Text> {
        // This requires specialization or Any-based downcasting
        // For now, return None and handle Text specially at call sites
        None
    }

    /// Get the root node ID
    pub fn root(&self) -> Option<LayoutNodeId> {
        self.root
    }

    /// Compute layout for the given viewport size
    pub fn compute_layout(&mut self, width: f32, height: f32) {
        if let Some(root) = self.root {
            self.layout_tree.compute_layout(
                root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
            );
        }
    }

    /// Get the layout tree for inspection
    pub fn layout(&self) -> &LayoutTree {
        &self.layout_tree
    }

    /// Render the entire tree to a DrawContext
    pub fn render(&self, ctx: &mut dyn DrawContext) {
        if let Some(root) = self.root {
            self.render_node(ctx, root, (0.0, 0.0));
        }
    }

    /// Render a single node and its children
    fn render_node(&self, ctx: &mut dyn DrawContext, node: LayoutNodeId, parent_offset: (f32, f32)) {
        let Some(bounds) = self.layout_tree.get_bounds(node, parent_offset) else {
            return;
        };

        let Some(render_node) = self.render_nodes.get(&node) else {
            return;
        };

        // Push transform for this node's position
        ctx.push_transform(Transform::translate(bounds.x, bounds.y));

        // Draw background if present
        if let Some(ref bg) = render_node.props.background {
            ctx.fill_rect(
                Rect::new(0.0, 0.0, bounds.width, bounds.height),
                render_node.props.border_radius,
                bg.clone(),
            );
        }

        // Render children (relative to this node's transform)
        for child_id in self.layout_tree.children(node) {
            self.render_node(ctx, child_id, (0.0, 0.0));
        }

        // Pop transform
        ctx.pop_transform();
    }

    /// Render with layer separation for glass effects
    ///
    /// This method renders elements in three passes:
    /// 1. Background elements (will be blurred behind glass)
    /// 2. Glass elements (blur effect applied)
    /// 3. Foreground elements (on top, not blurred)
    pub fn render_layered<F>(
        &self,
        background_ctx: &mut dyn DrawContext,
        mut glass_callback: F,
        foreground_ctx: &mut dyn DrawContext,
    ) where
        F: FnMut(LayoutNodeId, ElementBounds, &RenderProps),
    {
        if let Some(root) = self.root {
            // Pass 1: Background
            self.render_layer(background_ctx, root, (0.0, 0.0), RenderLayer::Background);

            // Pass 2: Glass (via callback)
            self.collect_glass_nodes(root, (0.0, 0.0), &mut glass_callback);

            // Pass 3: Foreground
            self.render_layer(foreground_ctx, root, (0.0, 0.0), RenderLayer::Foreground);
        }
    }

    /// Render only nodes in a specific layer
    fn render_layer(
        &self,
        ctx: &mut dyn DrawContext,
        node: LayoutNodeId,
        parent_offset: (f32, f32),
        target_layer: RenderLayer,
    ) {
        let Some(bounds) = self.layout_tree.get_bounds(node, parent_offset) else {
            return;
        };

        let Some(render_node) = self.render_nodes.get(&node) else {
            return;
        };

        // Only render if this node is in the target layer
        if render_node.props.layer == target_layer {
            ctx.push_transform(Transform::translate(bounds.x, bounds.y));

            if let Some(ref bg) = render_node.props.background {
                ctx.fill_rect(
                    Rect::new(0.0, 0.0, bounds.width, bounds.height),
                    render_node.props.border_radius,
                    bg.clone(),
                );
            }

            ctx.pop_transform();
        }

        // Always traverse children
        let new_offset = (parent_offset.0 + bounds.x, parent_offset.1 + bounds.y);
        for child_id in self.layout_tree.children(node) {
            self.render_layer(ctx, child_id, new_offset, target_layer);
        }
    }

    /// Collect glass nodes for the callback
    fn collect_glass_nodes<F>(
        &self,
        node: LayoutNodeId,
        parent_offset: (f32, f32),
        callback: &mut F,
    ) where
        F: FnMut(LayoutNodeId, ElementBounds, &RenderProps),
    {
        let Some(bounds) = self.layout_tree.get_bounds(node, parent_offset) else {
            return;
        };

        if let Some(render_node) = self.render_nodes.get(&node) {
            if render_node.props.layer == RenderLayer::Glass {
                callback(node, bounds, &render_node.props);
            }
        }

        // Traverse children
        let new_offset = (parent_offset.0 + bounds.x, parent_offset.1 + bounds.y);
        for child_id in self.layout_tree.children(node) {
            self.collect_glass_nodes(child_id, new_offset, callback);
        }
    }

    /// Get bounds for a specific node
    pub fn get_bounds(&self, node: LayoutNodeId) -> Option<ElementBounds> {
        self.layout_tree.get_bounds(node, (0.0, 0.0))
    }

    /// Get render node data
    pub fn get_render_node(&self, node: LayoutNodeId) -> Option<&RenderNode> {
        self.render_nodes.get(&node)
    }

    /// Iterate over all nodes with their bounds and render props
    pub fn iter_nodes(&self) -> impl Iterator<Item = (LayoutNodeId, &RenderNode)> {
        self.render_nodes.iter().map(|(&id, node)| (id, node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::div::div;

    #[test]
    fn test_render_tree_from_element() {
        let ui = div().w(100.0).h(100.0).child(div().w(50.0).h(50.0));

        let tree = RenderTree::from_element(&ui);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_compute_layout() {
        let ui = div()
            .w(200.0)
            .h(200.0)
            .flex_col()
            .child(div().h(50.0).w_full())
            .child(div().flex_grow().w_full());

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(200.0, 200.0);

        let root = tree.root().unwrap();
        let bounds = tree.get_bounds(root).unwrap();

        assert_eq!(bounds.width, 200.0);
        assert_eq!(bounds.height, 200.0);
    }
}
