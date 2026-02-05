//! Unified element styling
//!
//! Provides `ElementStyle` - a consistent style schema for all visual properties
//! that can be applied to layout elements. This enables:
//!
//! - Consistent API across `Div`, `StatefulDiv`, and other elements
//! - State-dependent styling with full property support
//! - Style composition and merging
//!
//! # Example
//!
//! ```ignore
//! use blinc_layout::prelude::*;
//! use blinc_core::Color;
//!
//! // Create a style
//! let style = ElementStyle::new()
//!     .bg(Color::BLUE)
//!     .rounded(8.0)
//!     .shadow_md()
//!     .scale(1.0);
//!
//! // Use with stateful elements
//! stateful_button()
//!     .idle(ElementStyle::new().bg(Color::BLUE))
//!     .hovered(ElementStyle::new().bg(Color::LIGHT_BLUE).scale(1.02))
//!     .pressed(ElementStyle::new().bg(Color::DARK_BLUE).scale(0.98));
//! ```

use blinc_core::{Brush, Color, CornerRadius, Shadow, Transform};
use blinc_theme::ThemeState;

use crate::css_parser::CssAnimation;
use crate::element::{GlassMaterial, Material, MetallicMaterial, RenderLayer, WoodMaterial};

/// Visual style properties for an element
///
/// All properties are optional - when merging styles, only set properties
/// will override. This enables state-specific styling where you only
/// override the properties that change for that state.
#[derive(Clone, Default, Debug)]
pub struct ElementStyle {
    /// Background brush (solid color, gradient, or glass)
    pub background: Option<Brush>,
    /// Corner radius
    pub corner_radius: Option<CornerRadius>,
    /// Drop shadow
    pub shadow: Option<Shadow>,
    /// Transform (scale, rotate, translate)
    pub transform: Option<Transform>,
    /// Material effect (glass, metallic, wood)
    pub material: Option<Material>,
    /// Render layer ordering
    pub render_layer: Option<RenderLayer>,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,
    /// CSS animation configuration (animation: name duration timing delay iteration-count direction fill-mode)
    pub animation: Option<CssAnimation>,
}

impl ElementStyle {
    /// Create a new empty style
    pub fn new() -> Self {
        Self::default()
    }

    // =========================================================================
    // Background
    // =========================================================================

    /// Set background color
    pub fn bg(mut self, color: impl Into<Brush>) -> Self {
        self.background = Some(color.into());
        self
    }

    /// Set background to a solid color
    pub fn bg_color(mut self, color: Color) -> Self {
        self.background = Some(Brush::Solid(color));
        self
    }

    /// Set background brush (for gradients, etc.)
    pub fn background(mut self, brush: Brush) -> Self {
        self.background = Some(brush);
        self
    }

    // =========================================================================
    // Corner Radius
    // =========================================================================

    /// Set uniform corner radius
    pub fn rounded(mut self, radius: f32) -> Self {
        self.corner_radius = Some(CornerRadius::uniform(radius));
        self
    }

    /// Set corner radius to full pill shape
    pub fn rounded_full(mut self) -> Self {
        self.corner_radius = Some(CornerRadius::uniform(9999.0));
        self
    }

    /// Set individual corner radii (top-left, top-right, bottom-right, bottom-left)
    pub fn rounded_corners(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.corner_radius = Some(CornerRadius::new(tl, tr, br, bl));
        self
    }

    /// Set corner radius directly
    pub fn corner_radius(mut self, radius: CornerRadius) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    // -------------------------------------------------------------------------
    // Theme-based corner radii
    // -------------------------------------------------------------------------

    /// Set corner radius to theme's small radius
    pub fn rounded_sm(self) -> Self {
        self.rounded(ThemeState::get().radii().radius_sm)
    }

    /// Set corner radius to theme's default radius
    pub fn rounded_default(self) -> Self {
        self.rounded(ThemeState::get().radii().radius_default)
    }

    /// Set corner radius to theme's medium radius
    pub fn rounded_md(self) -> Self {
        self.rounded(ThemeState::get().radii().radius_md)
    }

    /// Set corner radius to theme's large radius
    pub fn rounded_lg(self) -> Self {
        self.rounded(ThemeState::get().radii().radius_lg)
    }

    /// Set corner radius to theme's extra large radius
    pub fn rounded_xl(self) -> Self {
        self.rounded(ThemeState::get().radii().radius_xl)
    }

    /// Set corner radius to theme's 2xl radius
    pub fn rounded_2xl(self) -> Self {
        self.rounded(ThemeState::get().radii().radius_2xl)
    }

    /// Set corner radius to none (0)
    pub fn rounded_none(self) -> Self {
        self.rounded(0.0)
    }

    // =========================================================================
    // Shadow
    // =========================================================================

    /// Set drop shadow
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    /// Set shadow with parameters
    pub fn shadow_params(self, offset_x: f32, offset_y: f32, blur: f32, color: Color) -> Self {
        self.shadow(Shadow::new(offset_x, offset_y, blur, color))
    }

    /// Small shadow preset using theme colors
    pub fn shadow_sm(self) -> Self {
        self.shadow(ThemeState::get().shadows().shadow_sm.into())
    }

    /// Medium shadow preset using theme colors
    pub fn shadow_md(self) -> Self {
        self.shadow(ThemeState::get().shadows().shadow_md.into())
    }

    /// Large shadow preset using theme colors
    pub fn shadow_lg(self) -> Self {
        self.shadow(ThemeState::get().shadows().shadow_lg.into())
    }

    /// Extra large shadow preset using theme colors
    pub fn shadow_xl(self) -> Self {
        self.shadow(ThemeState::get().shadows().shadow_xl.into())
    }

    /// Explicitly clear shadow (override any inherited shadow)
    pub fn shadow_none(mut self) -> Self {
        // Use a fully transparent shadow to indicate "no shadow"
        self.shadow = Some(Shadow::new(0.0, 0.0, 0.0, Color::TRANSPARENT));
        self
    }

    // =========================================================================
    // Transform
    // =========================================================================

    /// Set transform
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Scale uniformly
    pub fn scale(self, factor: f32) -> Self {
        self.transform(Transform::scale(factor, factor))
    }

    /// Scale with different x and y factors
    pub fn scale_xy(self, sx: f32, sy: f32) -> Self {
        self.transform(Transform::scale(sx, sy))
    }

    /// Translate by x and y offset
    pub fn translate(self, x: f32, y: f32) -> Self {
        self.transform(Transform::translate(x, y))
    }

    /// Rotate by angle in radians
    pub fn rotate(self, angle: f32) -> Self {
        self.transform(Transform::rotate(angle))
    }

    /// Rotate by angle in degrees
    pub fn rotate_deg(self, degrees: f32) -> Self {
        self.rotate(degrees * std::f32::consts::PI / 180.0)
    }

    // =========================================================================
    // Material
    // =========================================================================

    /// Set material effect
    pub fn material(mut self, material: Material) -> Self {
        // Glass materials also set the render layer to Glass
        if matches!(material, Material::Glass(_)) {
            self.render_layer = Some(RenderLayer::Glass);
        }
        self.material = Some(material);
        self
    }

    /// Apply a visual effect
    pub fn effect(self, effect: impl Into<Material>) -> Self {
        self.material(effect.into())
    }

    /// Apply glass material with default settings
    pub fn glass(self) -> Self {
        self.material(Material::Glass(GlassMaterial::new()))
    }

    /// Apply glass material with custom settings
    pub fn glass_custom(self, glass: GlassMaterial) -> Self {
        self.material(Material::Glass(glass))
    }

    /// Apply metallic material with default settings
    pub fn metallic(self) -> Self {
        self.material(Material::Metallic(MetallicMaterial::new()))
    }

    /// Apply chrome metallic preset
    pub fn chrome(self) -> Self {
        self.material(Material::Metallic(MetallicMaterial::chrome()))
    }

    /// Apply gold metallic preset
    pub fn gold(self) -> Self {
        self.material(Material::Metallic(MetallicMaterial::gold()))
    }

    /// Apply wood material with default settings
    pub fn wood(self) -> Self {
        self.material(Material::Wood(WoodMaterial::new()))
    }

    // =========================================================================
    // Layer
    // =========================================================================

    /// Set render layer
    pub fn layer(mut self, layer: RenderLayer) -> Self {
        self.render_layer = Some(layer);
        self
    }

    /// Render in foreground layer
    pub fn foreground(self) -> Self {
        self.layer(RenderLayer::Foreground)
    }

    /// Render in background layer
    pub fn layer_background(self) -> Self {
        self.layer(RenderLayer::Background)
    }

    // =========================================================================
    // Opacity
    // =========================================================================

    /// Set opacity (0.0 = transparent, 1.0 = opaque)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Fully opaque
    pub fn opaque(self) -> Self {
        self.opacity(1.0)
    }

    /// Semi-transparent (50% opacity)
    pub fn translucent(self) -> Self {
        self.opacity(0.5)
    }

    /// Fully transparent
    pub fn transparent(self) -> Self {
        self.opacity(0.0)
    }

    // =========================================================================
    // Merging
    // =========================================================================

    /// Merge another style on top of this one
    ///
    /// Properties from `other` will override properties in `self` if they are set.
    /// Unset properties in `other` will not override.
    pub fn merge(&self, other: &ElementStyle) -> ElementStyle {
        ElementStyle {
            background: other.background.clone().or_else(|| self.background.clone()),
            corner_radius: other.corner_radius.or(self.corner_radius),
            shadow: other.shadow.clone().or_else(|| self.shadow.clone()),
            transform: other.transform.clone().or_else(|| self.transform.clone()),
            material: other.material.clone().or_else(|| self.material.clone()),
            render_layer: other.render_layer.or(self.render_layer),
            opacity: other.opacity.or(self.opacity),
            animation: other.animation.clone().or_else(|| self.animation.clone()),
        }
    }

    /// Check if any property is set
    pub fn is_empty(&self) -> bool {
        self.background.is_none()
            && self.corner_radius.is_none()
            && self.shadow.is_none()
            && self.transform.is_none()
            && self.material.is_none()
            && self.render_layer.is_none()
            && self.opacity.is_none()
            && self.animation.is_none()
    }

    // =========================================================================
    // Animation
    // =========================================================================

    /// Set CSS animation
    pub fn animation(mut self, animation: CssAnimation) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Set animation by name (requires stylesheet lookup later)
    pub fn animation_name(mut self, name: impl Into<String>) -> Self {
        let mut anim = self.animation.take().unwrap_or_default();
        anim.name = name.into();
        self.animation = Some(anim);
        self
    }

    /// Set animation duration in milliseconds
    pub fn animation_duration(mut self, duration_ms: u32) -> Self {
        let mut anim = self.animation.take().unwrap_or_default();
        anim.duration_ms = duration_ms;
        self.animation = Some(anim);
        self
    }
}

/// Create a new element style
pub fn style() -> ElementStyle {
    ElementStyle::new()
}

/// CSS-like macro for creating ElementStyle with CSS property names
///
/// Uses CSS property naming conventions (with hyphens parsed as separate tokens).
/// Provides a familiar syntax for developers coming from CSS/web development.
///
/// # Examples
///
/// ```ignore
/// use blinc_layout::prelude::*;
/// use blinc_core::Color;
///
/// // CSS-style properties (note: use spaces around hyphens)
/// let card = css! {
///     background: Color::WHITE;
///     border-radius: 8.0;
///     box-shadow: Shadow::new(0.0, 4.0, 8.0, Color::BLACK.with_alpha(0.2));
///     opacity: 0.9;
/// };
///
/// // Transform properties
/// let hover = css! {
///     transform: Transform::scale(1.05, 1.05);
///     opacity: 1.0;
/// };
///
/// // Material effects (Blinc extensions)
/// let glass_panel = css! {
///     background: Color::WHITE.with_alpha(0.1);
///     border-radius: 16.0;
///     backdrop-filter: glass;
/// };
///
/// // Animation
/// let animated = css! {
///     animation-name: "fade-in";
///     animation-duration: 300;
/// };
/// ```
///
/// # Supported Properties
///
/// ## Standard CSS Properties
/// - `background`: Color or Brush
/// - `border-radius`: f32 (uniform) or CornerRadius
/// - `box-shadow`: Shadow
/// - `opacity`: f32 (0.0-1.0)
/// - `transform`: Transform
///
/// ## Blinc Extensions
/// - `backdrop-filter`: `glass`, `metallic`, `chrome`, `gold`, `wood`
/// - `render-layer`: RenderLayer
///
/// ## Animation Properties
/// - `animation`: CssAnimation
/// - `animation-name`: String
/// - `animation-duration`: u32 (milliseconds)
#[macro_export]
macro_rules! css {
    // Empty style
    () => {
        $crate::element_style::ElementStyle::new()
    };

    // Main entry point - parse CSS properties (semicolon separated)
    ($($tokens:tt)*) => {{
        let mut __style = $crate::element_style::ElementStyle::new();
        $crate::css_impl!(__style; $($tokens)*);
        __style
    }};
}

/// Internal macro for parsing CSS properties
#[macro_export]
#[doc(hidden)]
macro_rules! css_impl {
    // Base case - no more tokens
    ($style:ident;) => {};

    // =========================================================================
    // Background (CSS: background)
    // =========================================================================
    ($style:ident; background: $value:expr; $($rest:tt)*) => {
        $style = $style.bg($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; background: $value:expr) => {
        $style = $style.bg($value);
    };

    // =========================================================================
    // Border Radius (CSS: border-radius)
    // =========================================================================
    ($style:ident; border-radius: $value:expr; $($rest:tt)*) => {
        $style = $style.rounded($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; border-radius: $value:expr) => {
        $style = $style.rounded($value);
    };

    // =========================================================================
    // Box Shadow (CSS: box-shadow)
    // Shadow presets must come BEFORE generic expr to match correctly
    // =========================================================================
    ($style:ident; box-shadow: sm; $($rest:tt)*) => {
        $style = $style.shadow_sm();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; box-shadow: sm) => {
        $style = $style.shadow_sm();
    };
    ($style:ident; box-shadow: md; $($rest:tt)*) => {
        $style = $style.shadow_md();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; box-shadow: md) => {
        $style = $style.shadow_md();
    };
    ($style:ident; box-shadow: lg; $($rest:tt)*) => {
        $style = $style.shadow_lg();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; box-shadow: lg) => {
        $style = $style.shadow_lg();
    };
    ($style:ident; box-shadow: xl; $($rest:tt)*) => {
        $style = $style.shadow_xl();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; box-shadow: xl) => {
        $style = $style.shadow_xl();
    };
    ($style:ident; box-shadow: none; $($rest:tt)*) => {
        $style = $style.shadow_none();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; box-shadow: none) => {
        $style = $style.shadow_none();
    };
    // Generic expression (must come after presets)
    ($style:ident; box-shadow: $value:expr; $($rest:tt)*) => {
        $style = $style.shadow($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; box-shadow: $value:expr) => {
        $style = $style.shadow($value);
    };

    // =========================================================================
    // Opacity (CSS: opacity)
    // =========================================================================
    ($style:ident; opacity: $value:expr; $($rest:tt)*) => {
        $style = $style.opacity($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; opacity: $value:expr) => {
        $style = $style.opacity($value);
    };

    // =========================================================================
    // Transform (CSS: transform)
    // =========================================================================
    ($style:ident; transform: $value:expr; $($rest:tt)*) => {
        $style = $style.transform($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; transform: $value:expr) => {
        $style = $style.transform($value);
    };
    // Scale shorthand
    ($style:ident; transform: scale($value:expr); $($rest:tt)*) => {
        $style = $style.scale($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; transform: scale($sx:expr, $sy:expr); $($rest:tt)*) => {
        $style = $style.scale_xy($sx, $sy);
        $crate::css_impl!($style; $($rest)*);
    };
    // Translate shorthand
    ($style:ident; transform: translate($x:expr, $y:expr); $($rest:tt)*) => {
        $style = $style.translate($x, $y);
        $crate::css_impl!($style; $($rest)*);
    };
    // Rotate shorthand (degrees)
    ($style:ident; transform: rotate($deg:expr); $($rest:tt)*) => {
        $style = $style.rotate_deg($deg);
        $crate::css_impl!($style; $($rest)*);
    };

    // =========================================================================
    // Backdrop Filter (Blinc extension for materials)
    // =========================================================================
    ($style:ident; backdrop-filter: glass; $($rest:tt)*) => {
        $style = $style.glass();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; backdrop-filter: glass) => {
        $style = $style.glass();
    };
    ($style:ident; backdrop-filter: metallic; $($rest:tt)*) => {
        $style = $style.metallic();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; backdrop-filter: chrome; $($rest:tt)*) => {
        $style = $style.chrome();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; backdrop-filter: gold; $($rest:tt)*) => {
        $style = $style.gold();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; backdrop-filter: wood; $($rest:tt)*) => {
        $style = $style.wood();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; backdrop-filter: $value:expr; $($rest:tt)*) => {
        $style = $style.material($value);
        $crate::css_impl!($style; $($rest)*);
    };

    // =========================================================================
    // Render Layer (Blinc extension)
    // =========================================================================
    ($style:ident; render-layer: foreground; $($rest:tt)*) => {
        $style = $style.foreground();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; render-layer: background; $($rest:tt)*) => {
        $style = $style.layer_background();
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; render-layer: $value:expr; $($rest:tt)*) => {
        $style = $style.layer($value);
        $crate::css_impl!($style; $($rest)*);
    };

    // =========================================================================
    // Animation Properties
    // =========================================================================
    ($style:ident; animation: $value:expr; $($rest:tt)*) => {
        $style = $style.animation($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; animation: $value:expr) => {
        $style = $style.animation($value);
    };
    ($style:ident; animation-name: $value:expr; $($rest:tt)*) => {
        $style = $style.animation_name($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; animation-name: $value:expr) => {
        $style = $style.animation_name($value);
    };
    ($style:ident; animation-duration: $value:expr; $($rest:tt)*) => {
        $style = $style.animation_duration($value);
        $crate::css_impl!($style; $($rest)*);
    };
    ($style:ident; animation-duration: $value:expr) => {
        $style = $style.animation_duration($value);
    };
}

/// Rust-friendly macro for creating ElementStyle with builder-like syntax
///
/// Uses Rust naming conventions (underscores instead of hyphens).
/// Comma-separated properties with colon syntax.
///
/// # Examples
///
/// ```ignore
/// use blinc_layout::prelude::*;
/// use blinc_core::Color;
///
/// // Basic usage with property: value syntax
/// let s = style! {
///     bg: Color::BLUE,
///     rounded: 8.0,
///     opacity: 0.9,
/// };
///
/// // Preset methods (no value needed)
/// let card = style! {
///     bg: Color::WHITE,
///     rounded_lg,
///     shadow_md,
/// };
///
/// // Transform shortcuts
/// let hover = style! {
///     scale: 1.05,
///     rotate_deg: 15.0,
///     translate: (10.0, 5.0),
/// };
///
/// // Material effects
/// let glass_panel = style! {
///     glass,
///     rounded: 16.0,
/// };
/// ```
#[macro_export]
macro_rules! style {
    // Empty style
    () => {
        $crate::element_style::ElementStyle::new()
    };

    // Main entry point - parse properties
    ($($tokens:tt)*) => {{
        let mut __style = $crate::element_style::ElementStyle::new();
        $crate::style_impl!(__style; $($tokens)*);
        __style
    }};
}

/// Internal macro for parsing style properties (Rust-style)
#[macro_export]
#[doc(hidden)]
macro_rules! style_impl {
    // Base case - no more tokens
    ($style:ident;) => {};

    // =========================================================================
    // Background properties
    // =========================================================================
    ($style:ident; bg: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.bg($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; background: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.background($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };

    // =========================================================================
    // Corner radius properties
    // =========================================================================
    ($style:ident; rounded: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.rounded($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rounded_corners: ($tl:expr, $tr:expr, $br:expr, $bl:expr) $(, $($rest:tt)*)?) => {
        $style = $style.rounded_corners($tl, $tr, $br, $bl);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    // Preset corner radii
    ($style:ident; rounded_sm $(, $($rest:tt)*)?) => {
        $style = $style.rounded_sm();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rounded_md $(, $($rest:tt)*)?) => {
        $style = $style.rounded_md();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rounded_lg $(, $($rest:tt)*)?) => {
        $style = $style.rounded_lg();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rounded_xl $(, $($rest:tt)*)?) => {
        $style = $style.rounded_xl();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rounded_2xl $(, $($rest:tt)*)?) => {
        $style = $style.rounded_2xl();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rounded_none $(, $($rest:tt)*)?) => {
        $style = $style.rounded_none();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rounded_full $(, $($rest:tt)*)?) => {
        $style = $style.rounded_full();
        $crate::style_impl!($style; $($($rest)*)?);
    };

    // =========================================================================
    // Shadow properties
    // =========================================================================
    ($style:ident; shadow: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.shadow($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; shadow_sm $(, $($rest:tt)*)?) => {
        $style = $style.shadow_sm();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; shadow_md $(, $($rest:tt)*)?) => {
        $style = $style.shadow_md();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; shadow_lg $(, $($rest:tt)*)?) => {
        $style = $style.shadow_lg();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; shadow_xl $(, $($rest:tt)*)?) => {
        $style = $style.shadow_xl();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; shadow_none $(, $($rest:tt)*)?) => {
        $style = $style.shadow_none();
        $crate::style_impl!($style; $($($rest)*)?);
    };

    // =========================================================================
    // Transform properties
    // =========================================================================
    ($style:ident; transform: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.transform($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; scale: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.scale($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; scale_xy: ($sx:expr, $sy:expr) $(, $($rest:tt)*)?) => {
        $style = $style.scale_xy($sx, $sy);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; translate: ($x:expr, $y:expr) $(, $($rest:tt)*)?) => {
        $style = $style.translate($x, $y);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rotate: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.rotate($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; rotate_deg: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.rotate_deg($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };

    // =========================================================================
    // Opacity properties
    // =========================================================================
    ($style:ident; opacity: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.opacity($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; opaque $(, $($rest:tt)*)?) => {
        $style = $style.opaque();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; translucent $(, $($rest:tt)*)?) => {
        $style = $style.translucent();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; transparent $(, $($rest:tt)*)?) => {
        $style = $style.transparent();
        $crate::style_impl!($style; $($($rest)*)?);
    };

    // =========================================================================
    // Material properties
    // =========================================================================
    ($style:ident; material: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.material($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; glass $(, $($rest:tt)*)?) => {
        $style = $style.glass();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; metallic $(, $($rest:tt)*)?) => {
        $style = $style.metallic();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; chrome $(, $($rest:tt)*)?) => {
        $style = $style.chrome();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; gold $(, $($rest:tt)*)?) => {
        $style = $style.gold();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; wood $(, $($rest:tt)*)?) => {
        $style = $style.wood();
        $crate::style_impl!($style; $($($rest)*)?);
    };

    // =========================================================================
    // Layer properties
    // =========================================================================
    ($style:ident; layer: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.layer($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; foreground $(, $($rest:tt)*)?) => {
        $style = $style.foreground();
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; layer_background $(, $($rest:tt)*)?) => {
        $style = $style.layer_background();
        $crate::style_impl!($style; $($($rest)*)?);
    };

    // =========================================================================
    // Animation properties
    // =========================================================================
    ($style:ident; animation: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.animation($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; animation_name: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.animation_name($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
    ($style:ident; animation_duration: $value:expr $(, $($rest:tt)*)?) => {
        $style = $style.animation_duration($value);
        $crate::style_impl!($style; $($($rest)*)?);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_builder() {
        // Initialize theme (required for shadow_md which uses theme)
        ThemeState::init_default();

        let s = style().bg(Color::BLUE).rounded(8.0).shadow_md().scale(1.05);

        assert!(s.background.is_some());
        assert!(s.corner_radius.is_some());
        assert!(s.shadow.is_some());
        assert!(s.transform.is_some());
    }

    #[test]
    fn test_style_merge() {
        // Initialize theme (required for shadow_sm which uses theme)
        ThemeState::init_default();

        let base = style().bg(Color::BLUE).rounded(8.0).shadow_sm();

        let hover = style().bg(Color::GREEN).scale(1.02);

        let merged = base.merge(&hover);

        // Background should be overridden
        assert!(matches!(merged.background, Some(Brush::Solid(c)) if c == Color::GREEN));
        // Corner radius should be preserved from base
        assert!(merged.corner_radius.is_some());
        // Shadow should be preserved from base
        assert!(merged.shadow.is_some());
        // Transform should come from hover
        assert!(merged.transform.is_some());
    }

    #[test]
    fn test_style_empty() {
        let empty = ElementStyle::new();
        assert!(empty.is_empty());

        let non_empty = style().bg(Color::RED);
        assert!(!non_empty.is_empty());
    }

    // =========================================================================
    // style! macro tests
    // =========================================================================

    #[test]
    fn test_style_macro_empty() {
        let s = style!();
        assert!(s.is_empty());
    }

    #[test]
    fn test_style_macro_basic() {
        ThemeState::init_default();

        let s = style! {
            bg: Color::BLUE,
            rounded: 8.0,
            opacity: 0.9,
        };

        assert!(matches!(s.background, Some(Brush::Solid(c)) if c == Color::BLUE));
        assert!(s.corner_radius.is_some());
        assert_eq!(s.opacity, Some(0.9));
    }

    #[test]
    fn test_style_macro_presets() {
        ThemeState::init_default();

        let s = style! {
            bg: Color::WHITE,
            rounded_lg,
            shadow_md,
        };

        assert!(s.background.is_some());
        assert!(s.corner_radius.is_some());
        assert!(s.shadow.is_some());
    }

    #[test]
    fn test_style_macro_transforms() {
        let s = style! {
            scale: 1.05,
        };
        assert!(s.transform.is_some());

        let s2 = style! {
            translate: (10.0, 20.0),
        };
        assert!(s2.transform.is_some());

        let s3 = style! {
            rotate_deg: 45.0,
        };
        assert!(s3.transform.is_some());

        let s4 = style! {
            scale_xy: (1.1, 0.9),
        };
        assert!(s4.transform.is_some());
    }

    #[test]
    fn test_style_macro_materials() {
        let s = style! {
            glass,
            rounded: 16.0,
        };

        assert!(s.material.is_some());
        assert!(s.corner_radius.is_some());
        // Glass sets render layer to Glass
        assert!(s.render_layer.is_some());
    }

    #[test]
    fn test_style_macro_opacity_presets() {
        let s1 = style! { opaque };
        assert_eq!(s1.opacity, Some(1.0));

        let s2 = style! { translucent };
        assert_eq!(s2.opacity, Some(0.5));

        let s3 = style! { transparent };
        assert_eq!(s3.opacity, Some(0.0));
    }

    #[test]
    fn test_style_macro_combined() {
        ThemeState::init_default();

        // Test combining multiple properties
        let card_style = style! {
            bg: Color::WHITE,
            rounded_lg,
            shadow_md,
            opacity: 0.95,
            scale: 1.0,
        };

        assert!(card_style.background.is_some());
        assert!(card_style.corner_radius.is_some());
        assert!(card_style.shadow.is_some());
        assert_eq!(card_style.opacity, Some(0.95));
        assert!(card_style.transform.is_some());
    }

    #[test]
    fn test_style_macro_rounded_variants() {
        ThemeState::init_default();

        let s1 = style! { rounded_sm };
        assert!(s1.corner_radius.is_some());

        let s2 = style! { rounded_md };
        assert!(s2.corner_radius.is_some());

        let s3 = style! { rounded_xl };
        assert!(s3.corner_radius.is_some());

        let s4 = style! { rounded_full };
        assert!(s4.corner_radius.is_some());

        let s5 = style! { rounded_none };
        assert!(s5.corner_radius.is_some());
    }

    #[test]
    fn test_style_macro_shadow_variants() {
        ThemeState::init_default();

        let s1 = style! { shadow_sm };
        assert!(s1.shadow.is_some());

        let s2 = style! { shadow_lg };
        assert!(s2.shadow.is_some());

        let s3 = style! { shadow_xl };
        assert!(s3.shadow.is_some());

        let s4 = style! { shadow_none };
        assert!(s4.shadow.is_some()); // shadow_none sets a transparent shadow
    }

    #[test]
    fn test_style_macro_material_variants() {
        let s1 = style! { metallic };
        assert!(s1.material.is_some());

        let s2 = style! { chrome };
        assert!(s2.material.is_some());

        let s3 = style! { gold };
        assert!(s3.material.is_some());

        let s4 = style! { wood };
        assert!(s4.material.is_some());
    }

    #[test]
    fn test_style_macro_layer() {
        let s1 = style! { foreground };
        assert!(s1.render_layer.is_some());

        let s2 = style! { layer_background };
        assert!(s2.render_layer.is_some());
    }

    #[test]
    fn test_style_macro_rounded_corners() {
        let s = style! {
            rounded_corners: (8.0, 8.0, 0.0, 0.0),
        };
        assert!(s.corner_radius.is_some());
        let cr = s.corner_radius.unwrap();
        assert_eq!(cr.top_left, 8.0);
        assert_eq!(cr.top_right, 8.0);
        assert_eq!(cr.bottom_right, 0.0);
        assert_eq!(cr.bottom_left, 0.0);
    }

    // =========================================================================
    // css! macro tests - CSS property name compatibility
    // =========================================================================

    #[test]
    fn test_css_macro_empty() {
        let s = css!();
        assert!(s.is_empty());
    }

    #[test]
    fn test_css_macro_basic() {
        // Uses CSS property names with semicolon separators
        let s = css! {
            background: Color::BLUE;
            border-radius: 8.0;
            opacity: 0.9;
        };

        assert!(matches!(s.background, Some(Brush::Solid(c)) if c == Color::BLUE));
        assert!(s.corner_radius.is_some());
        assert_eq!(s.opacity, Some(0.9));
    }

    #[test]
    fn test_css_macro_shadow() {
        ThemeState::init_default();

        let s = css! {
            box-shadow: md;
        };
        assert!(s.shadow.is_some());

        let s2 = css! {
            box-shadow: Shadow::new(0.0, 4.0, 8.0, Color::BLACK);
        };
        assert!(s2.shadow.is_some());
    }

    #[test]
    fn test_css_macro_transform() {
        let s = css! {
            transform: Transform::scale(1.05, 1.05);
        };
        assert!(s.transform.is_some());
    }

    #[test]
    fn test_css_macro_backdrop_filter() {
        // Blinc extension for materials
        let s = css! {
            backdrop-filter: glass;
        };
        assert!(s.material.is_some());
        assert!(s.render_layer.is_some()); // Glass sets render layer
    }

    #[test]
    fn test_css_macro_combined() {
        ThemeState::init_default();

        // Full CSS-like card style
        let card = css! {
            background: Color::WHITE;
            border-radius: 12.0;
            box-shadow: lg;
            opacity: 0.95;
        };

        assert!(card.background.is_some());
        assert!(card.corner_radius.is_some());
        assert!(card.shadow.is_some());
        assert_eq!(card.opacity, Some(0.95));
    }

    #[test]
    fn test_css_macro_animation() {
        let s = css! {
            animation-name: "fade-in";
            animation-duration: 300;
        };

        assert!(s.animation.is_some());
        let anim = s.animation.unwrap();
        assert_eq!(anim.name, "fade-in");
        assert_eq!(anim.duration_ms, 300);
    }

    #[test]
    fn test_css_and_style_macros_produce_same_result() {
        // Both macros should produce equivalent ElementStyle for same properties
        let from_css = css! {
            background: Color::RED;
            border-radius: 10.0;
            opacity: 0.8;
        };

        let from_style = style! {
            bg: Color::RED,
            rounded: 10.0,
            opacity: 0.8,
        };

        // Same background
        assert!(matches!(from_css.background, Some(Brush::Solid(c)) if c == Color::RED));
        assert!(matches!(from_style.background, Some(Brush::Solid(c)) if c == Color::RED));

        // Same corner radius
        assert_eq!(from_css.corner_radius, from_style.corner_radius);

        // Same opacity
        assert_eq!(from_css.opacity, from_style.opacity);
    }
}
