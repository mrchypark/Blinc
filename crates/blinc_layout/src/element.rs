//! Element types and traits for layout-driven UI
//!
//! Provides the core abstractions for building layout trees that can be
//! rendered via the DrawContext API.

use blinc_core::{Brush, Color, CornerRadius, Rect};
use taffy::Layout;

use crate::tree::LayoutNodeId;

/// Computed layout bounds for an element after layout computation
#[derive(Clone, Copy, Debug, Default)]
pub struct ElementBounds {
    /// X position relative to parent
    pub x: f32,
    /// Y position relative to parent
    pub y: f32,
    /// Computed width
    pub width: f32,
    /// Computed height
    pub height: f32,
}

impl ElementBounds {
    /// Create bounds from a Taffy Layout with parent offset
    pub fn from_layout(layout: &Layout, parent_offset: (f32, f32)) -> Self {
        Self {
            x: parent_offset.0 + layout.location.x,
            y: parent_offset.1 + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }

    /// Create bounds at origin with given size
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Convert to a blinc_core Rect
    pub fn to_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    /// Get bounds relative to self (origin at 0,0)
    pub fn local(&self) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: self.width,
            height: self.height,
        }
    }
}

/// Render layer for separating elements in glass-effect rendering
///
/// When using glass/vibrancy effects, elements need to be rendered in
/// different passes:
/// - Background elements are rendered first and get blurred behind glass
/// - Glass elements render the glass effect itself
/// - Foreground elements render on top without being blurred
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderLayer {
    /// Rendered behind glass (will be blurred)
    #[default]
    Background,
    /// Rendered as a glass element (blur effect applied)
    Glass,
    /// Rendered on top of glass (not blurred)
    Foreground,
}

/// Visual properties for rendering an element
#[derive(Clone, Default)]
pub struct RenderProps {
    /// Background fill (solid color or gradient)
    pub background: Option<Brush>,
    /// Corner radius for rounded rectangles
    pub border_radius: CornerRadius,
    /// Which layer this element renders in
    pub layer: RenderLayer,
    /// Node ID for looking up children
    pub node_id: Option<LayoutNodeId>,
}

impl RenderProps {
    /// Create new render properties
    pub fn new() -> Self {
        Self::default()
    }

    /// Set background brush
    pub fn with_background(mut self, brush: impl Into<Brush>) -> Self {
        self.background = Some(brush.into());
        self
    }

    /// Set background color
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.background = Some(Brush::Solid(color));
        self
    }

    /// Set corner radius
    pub fn with_border_radius(mut self, radius: CornerRadius) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set uniform corner radius
    pub fn with_rounded(mut self, radius: f32) -> Self {
        self.border_radius = CornerRadius::uniform(radius);
        self
    }

    /// Set render layer
    pub fn with_layer(mut self, layer: RenderLayer) -> Self {
        self.layer = layer;
        self
    }

    /// Set node ID
    pub fn with_node_id(mut self, id: LayoutNodeId) -> Self {
        self.node_id = Some(id);
        self
    }
}
