//! Image element builder
//!
//! Provides a builder for image elements that participate in layout:
//! ```rust
//! use blinc_layout::prelude::*;
//!
//! let photo = img("https://example.com/photo.jpg")
//!     .size(200.0, 150.0)
//!     .cover()
//!     .rounded(8.0);
//! ```

use blinc_core::{Shadow, Transform};
use taffy::prelude::*;

use crate::div::{ElementBuilder, ElementTypeId, ImageRenderInfo};
use crate::element::{RenderLayer, RenderProps};
use crate::tree::{LayoutNodeId, LayoutTree};

// ============================================================================
// Object Fit (mirroring blinc_image for layout purposes)
// ============================================================================

/// How an image should fit within its container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectFit {
    /// Fill the container completely, cropping if necessary (maintains aspect ratio)
    #[default]
    Cover,
    /// Fit entirely within the container (maintains aspect ratio, may letterbox)
    Contain,
    /// Stretch to fill the container (ignores aspect ratio)
    Fill,
    /// Scale down only if larger than container (maintains aspect ratio)
    ScaleDown,
    /// No scaling, display at original size
    None,
}

impl ObjectFit {
    fn to_u8(self) -> u8 {
        match self {
            ObjectFit::Cover => 0,
            ObjectFit::Contain => 1,
            ObjectFit::Fill => 2,
            ObjectFit::ScaleDown => 3,
            ObjectFit::None => 4,
        }
    }
}

// ============================================================================
// Object Position
// ============================================================================

/// Image alignment within its container
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ObjectPosition {
    /// Horizontal alignment (0.0 = left, 0.5 = center, 1.0 = right)
    pub x: f32,
    /// Vertical alignment (0.0 = top, 0.5 = center, 1.0 = bottom)
    pub y: f32,
}

impl ObjectPosition {
    pub const TOP_LEFT: Self = Self { x: 0.0, y: 0.0 };
    pub const TOP_CENTER: Self = Self { x: 0.5, y: 0.0 };
    pub const TOP_RIGHT: Self = Self { x: 1.0, y: 0.0 };
    pub const CENTER_LEFT: Self = Self { x: 0.0, y: 0.5 };
    pub const CENTER: Self = Self { x: 0.5, y: 0.5 };
    pub const CENTER_RIGHT: Self = Self { x: 1.0, y: 0.5 };
    pub const BOTTOM_LEFT: Self = Self { x: 0.0, y: 1.0 };
    pub const BOTTOM_CENTER: Self = Self { x: 0.5, y: 1.0 };
    pub const BOTTOM_RIGHT: Self = Self { x: 1.0, y: 1.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

// ============================================================================
// Image Filter
// ============================================================================

/// Image filter effects
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ImageFilter {
    /// Grayscale amount (0.0 = none, 1.0 = full grayscale)
    pub grayscale: f32,
    /// Sepia amount (0.0 = none, 1.0 = full sepia)
    pub sepia: f32,
    /// Brightness multiplier (1.0 = normal)
    pub brightness: f32,
    /// Contrast multiplier (1.0 = normal)
    pub contrast: f32,
    /// Saturation multiplier (1.0 = normal)
    pub saturate: f32,
    /// Hue rotation in degrees (0-360)
    pub hue_rotate: f32,
    /// Invert amount (0.0 = none, 1.0 = full invert)
    pub invert: f32,
    /// Blur radius in pixels
    pub blur: f32,
}

impl ImageFilter {
    /// Create a filter with no effects (identity)
    pub fn none() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturate: 1.0,
            ..Default::default()
        }
    }

    fn to_array(&self) -> [f32; 8] {
        [
            self.grayscale,
            self.sepia,
            self.brightness,
            self.contrast,
            self.saturate,
            self.hue_rotate,
            self.invert,
            self.blur,
        ]
    }
}

// ============================================================================
// Image Element
// ============================================================================

/// An image element builder
pub struct Image {
    /// Image source (file path, URL, base64, or data URI)
    source: String,
    /// Width in pixels
    width: f32,
    /// Height in pixels
    height: f32,
    /// Object fit mode
    object_fit: ObjectFit,
    /// Object position
    object_position: ObjectPosition,
    /// Opacity
    opacity: f32,
    /// Border radius for rounded corners
    border_radius: f32,
    /// Tint color [r, g, b, a]
    tint: [f32; 4],
    /// Image filter
    filter: ImageFilter,
    /// Taffy style for layout
    style: Style,
    /// Render layer
    render_layer: RenderLayer,
    /// Drop shadow
    shadow: Option<Shadow>,
    /// Transform
    transform: Option<Transform>,
}

impl Image {
    /// Create a new image element from a source
    ///
    /// Source can be:
    /// - File path: `path/to/image.png`
    /// - URL: `https://example.com/image.jpg`
    /// - Base64: `data:image/png;base64,iVBORw0KGgo...`
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            width: 100.0,
            height: 100.0,
            object_fit: ObjectFit::default(),
            object_position: ObjectPosition::CENTER,
            opacity: 1.0,
            border_radius: 0.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            filter: ImageFilter::none(),
            style: Style {
                size: taffy::Size {
                    width: Dimension::Length(100.0),
                    height: Dimension::Length(100.0),
                },
                ..Default::default()
            },
            render_layer: RenderLayer::default(),
            shadow: None,
            transform: None,
        }
    }

    // =========================================================================
    // Size
    // =========================================================================

    /// Set the size (width and height)
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.style.size.width = Dimension::Length(width);
        self.style.size.height = Dimension::Length(height);
        self
    }

    /// Set square size (width = height)
    pub fn square(mut self, size: f32) -> Self {
        self.width = size;
        self.height = size;
        self.style.size.width = Dimension::Length(size);
        self.style.size.height = Dimension::Length(size);
        self
    }

    /// Set width
    pub fn w(mut self, width: f32) -> Self {
        self.width = width;
        self.style.size.width = Dimension::Length(width);
        self
    }

    /// Set height
    pub fn h(mut self, height: f32) -> Self {
        self.height = height;
        self.style.size.height = Dimension::Length(height);
        self
    }

    /// Set width to 100%
    pub fn w_full(mut self) -> Self {
        self.style.size.width = Dimension::Percent(1.0);
        self
    }

    /// Set height to 100%
    pub fn h_full(mut self) -> Self {
        self.style.size.height = Dimension::Percent(1.0);
        self
    }

    // =========================================================================
    // Object Fit (CSS object-fit equivalent)
    // =========================================================================

    /// Set object-fit mode
    pub fn fit(mut self, fit: ObjectFit) -> Self {
        self.object_fit = fit;
        self
    }

    /// Fill the container, cropping if necessary (object-fit: cover)
    pub fn cover(self) -> Self {
        self.fit(ObjectFit::Cover)
    }

    /// Fit within container, letterboxing if necessary (object-fit: contain)
    pub fn contain(self) -> Self {
        self.fit(ObjectFit::Contain)
    }

    /// Stretch to fill, ignoring aspect ratio (object-fit: fill)
    pub fn fill(self) -> Self {
        self.fit(ObjectFit::Fill)
    }

    /// Scale down only if larger (object-fit: scale-down)
    pub fn scale_down(self) -> Self {
        self.fit(ObjectFit::ScaleDown)
    }

    /// No scaling (object-fit: none)
    pub fn no_scale(self) -> Self {
        self.fit(ObjectFit::None)
    }

    // =========================================================================
    // Object Position (CSS object-position equivalent)
    // =========================================================================

    /// Set object position
    pub fn position(mut self, position: ObjectPosition) -> Self {
        self.object_position = position;
        self
    }

    /// Position at custom x, y (0.0-1.0)
    pub fn position_xy(mut self, x: f32, y: f32) -> Self {
        self.object_position = ObjectPosition::new(x, y);
        self
    }

    /// Position at top-left
    pub fn top_left(self) -> Self {
        self.position(ObjectPosition::TOP_LEFT)
    }

    /// Position at top-center
    pub fn top_center(self) -> Self {
        self.position(ObjectPosition::TOP_CENTER)
    }

    /// Position at top-right
    pub fn top_right(self) -> Self {
        self.position(ObjectPosition::TOP_RIGHT)
    }

    /// Position at center-left
    pub fn center_left(self) -> Self {
        self.position(ObjectPosition::CENTER_LEFT)
    }

    /// Position at center (default)
    pub fn center(self) -> Self {
        self.position(ObjectPosition::CENTER)
    }

    /// Position at center-right
    pub fn center_right(self) -> Self {
        self.position(ObjectPosition::CENTER_RIGHT)
    }

    /// Position at bottom-left
    pub fn bottom_left(self) -> Self {
        self.position(ObjectPosition::BOTTOM_LEFT)
    }

    /// Position at bottom-center
    pub fn bottom_center(self) -> Self {
        self.position(ObjectPosition::BOTTOM_CENTER)
    }

    /// Position at bottom-right
    pub fn bottom_right(self) -> Self {
        self.position(ObjectPosition::BOTTOM_RIGHT)
    }

    // =========================================================================
    // Visual Properties
    // =========================================================================

    /// Set opacity (0.0 = transparent, 1.0 = opaque)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set border radius for rounded corners
    pub fn rounded(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Make fully circular (radius = min(width, height) / 2)
    pub fn circular(mut self) -> Self {
        self.border_radius = self.width.min(self.height) / 2.0;
        self
    }

    /// Set tint color (multiplied with image colors)
    pub fn tint(mut self, r: f32, g: f32, b: f32) -> Self {
        self.tint = [r, g, b, self.tint[3]];
        self
    }

    /// Set tint with alpha
    pub fn tint_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.tint = [r, g, b, a];
        self
    }

    // =========================================================================
    // Filters (CSS filter equivalent)
    // =========================================================================

    /// Set complete filter
    pub fn filter(mut self, filter: ImageFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Apply grayscale (0.0 = none, 1.0 = full grayscale)
    pub fn grayscale(mut self, amount: f32) -> Self {
        self.filter.grayscale = amount.clamp(0.0, 1.0);
        self
    }

    /// Apply sepia (0.0 = none, 1.0 = full sepia)
    pub fn sepia(mut self, amount: f32) -> Self {
        self.filter.sepia = amount.clamp(0.0, 1.0);
        self
    }

    /// Adjust brightness (1.0 = normal)
    pub fn brightness(mut self, amount: f32) -> Self {
        self.filter.brightness = amount.max(0.0);
        self
    }

    /// Adjust contrast (1.0 = normal)
    pub fn contrast(mut self, amount: f32) -> Self {
        self.filter.contrast = amount.max(0.0);
        self
    }

    /// Adjust saturation (1.0 = normal, 0.0 = grayscale)
    pub fn saturate(mut self, amount: f32) -> Self {
        self.filter.saturate = amount.max(0.0);
        self
    }

    /// Rotate hue (in degrees)
    pub fn hue_rotate(mut self, degrees: f32) -> Self {
        self.filter.hue_rotate = degrees % 360.0;
        self
    }

    /// Invert colors (0.0 = none, 1.0 = full invert)
    pub fn invert(mut self, amount: f32) -> Self {
        self.filter.invert = amount.clamp(0.0, 1.0);
        self
    }

    /// Apply blur (radius in pixels)
    pub fn blur(mut self, radius: f32) -> Self {
        self.filter.blur = radius.max(0.0);
        self
    }

    // =========================================================================
    // Render Layer
    // =========================================================================

    /// Set the render layer
    pub fn layer(mut self, layer: RenderLayer) -> Self {
        self.render_layer = layer;
        self
    }

    /// Render in foreground (on top of glass)
    pub fn foreground(self) -> Self {
        self.layer(RenderLayer::Foreground)
    }

    // =========================================================================
    // Layout Properties
    // =========================================================================

    /// Set margin on all sides (in 4px units)
    pub fn m(mut self, units: f32) -> Self {
        let px = LengthPercentageAuto::Length(units * 4.0);
        self.style.margin = Rect {
            left: px,
            right: px,
            top: px,
            bottom: px,
        };
        self
    }

    /// Set horizontal margin (in 4px units)
    pub fn mx(mut self, units: f32) -> Self {
        let px = LengthPercentageAuto::Length(units * 4.0);
        self.style.margin.left = px;
        self.style.margin.right = px;
        self
    }

    /// Set vertical margin (in 4px units)
    pub fn my(mut self, units: f32) -> Self {
        let px = LengthPercentageAuto::Length(units * 4.0);
        self.style.margin.top = px;
        self.style.margin.bottom = px;
        self
    }

    /// Set flex-grow
    pub fn flex_grow(mut self) -> Self {
        self.style.flex_grow = 1.0;
        self
    }

    /// Set flex-shrink
    pub fn flex_shrink(mut self) -> Self {
        self.style.flex_shrink = 1.0;
        self
    }

    /// Align self center
    pub fn self_center(mut self) -> Self {
        self.style.align_self = Some(AlignSelf::Center);
        self
    }

    // =========================================================================
    // Shadow
    // =========================================================================

    /// Apply a drop shadow
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    /// Apply a drop shadow with parameters
    pub fn shadow_params(
        self,
        offset_x: f32,
        offset_y: f32,
        blur: f32,
        color: blinc_core::Color,
    ) -> Self {
        self.shadow(Shadow::new(offset_x, offset_y, blur, color))
    }

    // =========================================================================
    // Transform
    // =========================================================================

    /// Apply a transform
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Translate by x and y
    pub fn translate(self, x: f32, y: f32) -> Self {
        self.transform(Transform::translate(x, y))
    }

    /// Scale uniformly
    pub fn scale(self, factor: f32) -> Self {
        self.transform(Transform::scale(factor, factor))
    }

    /// Rotate by angle in radians
    pub fn rotate(self, angle: f32) -> Self {
        self.transform(Transform::rotate(angle))
    }

    // =========================================================================
    // Getters
    // =========================================================================

    /// Get the image source
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the width
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get the height
    pub fn height(&self) -> f32 {
        self.height
    }
}

impl ElementBuilder for Image {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        tree.create_node(self.style.clone())
    }

    fn render_props(&self) -> RenderProps {
        RenderProps {
            background: None,
            border_radius: blinc_core::CornerRadius::uniform(self.border_radius),
            layer: self.render_layer,
            material: None,
            node_id: None,
            shadow: self.shadow,
            transform: self.transform.clone(),
            opacity: self.opacity,
            clips_content: false,
            motion: None,
            is_stack_layer: false,
        }
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        &[] // Image has no children
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementTypeId::Image
    }

    fn image_render_info(&self) -> Option<ImageRenderInfo> {
        Some(ImageRenderInfo {
            source: self.source.clone(),
            object_fit: self.object_fit.to_u8(),
            object_position: [self.object_position.x, self.object_position.y],
            opacity: self.opacity,
            border_radius: self.border_radius,
            tint: self.tint,
            filter: self.filter.to_array(),
        })
    }
}

/// Convenience function to create a new image element
///
/// Source can be:
/// - File path: `path/to/image.png`
/// - URL: `https://example.com/image.jpg`
/// - Base64 data URI: `data:image/png;base64,iVBORw0KGgo...`
pub fn img(source: impl Into<String>) -> Image {
    Image::new(source)
}

/// Alias for img() for those who prefer the full name
pub fn image(source: impl Into<String>) -> Image {
    Image::new(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_builder() {
        let i = img("test.png")
            .size(200.0, 150.0)
            .cover()
            .rounded(8.0)
            .opacity(0.9);

        assert_eq!(i.source(), "test.png");
        assert_eq!(i.width(), 200.0);
        assert_eq!(i.height(), 150.0);
    }

    #[test]
    fn test_image_filters() {
        let i = img("test.png").grayscale(0.5).brightness(1.2).contrast(1.1);

        let info = i.image_render_info().unwrap();
        assert_eq!(info.filter[0], 0.5); // grayscale
        assert_eq!(info.filter[2], 1.2); // brightness
        assert_eq!(info.filter[3], 1.1); // contrast
    }

    #[test]
    fn test_image_build() {
        let i = img("test.png");

        let mut tree = LayoutTree::new();
        let _node = i.build(&mut tree);

        assert_eq!(tree.len(), 1);
    }
}
