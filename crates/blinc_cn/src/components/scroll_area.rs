//! Scroll Area component - styled scrollable container with customizable scrollbar
//!
//! A themed scroll container that wraps `blinc_layout::scroll()` with the built-in
//! scrollbar. Supports various scrollbar visibility modes and auto-dismiss.
//!
//! # Example
//!
//! ```ignore
//! use blinc_cn::prelude::*;
//!
//! // Basic scroll area with auto-dismissing scrollbar
//! cn::scroll_area()
//!     .h(400.0)
//!     .child(
//!         div().flex_col().gap(8.0)
//!             .child(text("Item 1"))
//!             .child(text("Item 2"))
//!             // ... many items
//!     )
//!
//! // Always show scrollbar
//! cn::scroll_area()
//!     .scrollbar(ScrollbarVisibility::Always)
//!     .h(300.0)
//!     .child(content)
//!
//! // Horizontal scroll
//! cn::scroll_area()
//!     .horizontal()
//!     .w(400.0)
//!     .child(wide_content)
//!
//! // Custom scrollbar styling
//! cn::scroll_area()
//!     .scrollbar_width(8.0)
//!     .scrollbar_color(Color::GRAY)
//!     .h(400.0)
//!     .child(content)
//! ```

use std::cell::{OnceCell, RefCell};

use crate::components::responsive::{current_device_class, DeviceClass};
use blinc_core::Color;
use blinc_layout::element::RenderProps;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_layout::widgets::scroll::{
    scroll, Scroll, ScrollDirection, ScrollbarSize,
    ScrollbarVisibility as LayoutScrollbarVisibility,
};
use blinc_theme::{ColorToken, ThemeState};

const LEGACY_FALLBACK_WIDTH: f32 = 300.0;
const LEGACY_FALLBACK_HEIGHT: f32 = 400.0;

/// Scrollbar visibility modes
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    /// Always show scrollbar (like classic Windows style)
    Always,
    /// Show scrollbar only when hovering over the scroll area
    Hover,
    /// Show when scrolling, auto-dismiss after inactivity (like macOS)
    #[default]
    Auto,
    /// Never show scrollbar (content still scrollable)
    Never,
}

impl ScrollbarVisibility {
    /// Convert to the layout crate's ScrollbarVisibility
    fn to_layout(self) -> LayoutScrollbarVisibility {
        match self {
            ScrollbarVisibility::Always => LayoutScrollbarVisibility::Always,
            ScrollbarVisibility::Hover => LayoutScrollbarVisibility::Hover,
            ScrollbarVisibility::Auto => LayoutScrollbarVisibility::Auto,
            ScrollbarVisibility::Never => LayoutScrollbarVisibility::Never,
        }
    }
}

/// Scroll area size presets
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAreaSize {
    /// Small scrollbar (4px width)
    Small,
    /// Medium scrollbar (6px width)
    #[default]
    Medium,
    /// Large scrollbar (10px width)
    Large,
}

impl ScrollAreaSize {
    #[allow(dead_code)]
    fn scrollbar_width(&self) -> f32 {
        match self {
            ScrollAreaSize::Small => 4.0,
            ScrollAreaSize::Medium => 6.0,
            ScrollAreaSize::Large => 10.0,
        }
    }

    /// Convert to the layout crate's ScrollbarSize
    fn to_layout(self) -> ScrollbarSize {
        match self {
            ScrollAreaSize::Small => ScrollbarSize::Thin,
            ScrollAreaSize::Medium => ScrollbarSize::Normal,
            ScrollAreaSize::Large => ScrollbarSize::Wide,
        }
    }
}

/// Configuration for scroll area
struct ScrollAreaConfig {
    /// Scrollbar visibility mode
    visibility: ScrollbarVisibility,
    /// Scroll direction
    direction: ScrollDirection,
    /// Custom scrollbar width (overrides size preset)
    scrollbar_width: Option<f32>,
    /// Scrollbar size preset
    size: ScrollAreaSize,
    /// Thumb color (uses theme if None)
    thumb_color: Option<Color>,
    /// Track color (uses theme if None)
    track_color: Option<Color>,
    /// Viewport dimensions
    width: Option<f32>,
    height: Option<f32>,
    /// Enable responsive viewport defaults (Tailwind-style behavior)
    responsive: bool,
    /// Enable bounce physics
    bounce: bool,
    /// Content builder
    content: Option<Div>,
    /// Corner radius
    rounded: Option<f32>,
    /// Background color
    bg: Option<Color>,
}

impl Default for ScrollAreaConfig {
    fn default() -> Self {
        Self {
            visibility: ScrollbarVisibility::default(),
            direction: ScrollDirection::Vertical,
            scrollbar_width: None,
            size: ScrollAreaSize::default(),
            thumb_color: None,
            track_color: None,
            width: None,
            height: None,
            responsive: true,
            bounce: true,
            content: None,
            rounded: None,
            bg: None,
        }
    }
}

/// Built scroll area using the underlying scroll widget
struct BuiltScrollArea {
    inner: Scroll,
}

impl BuiltScrollArea {
    fn from_config(config: ScrollAreaConfig) -> Self {
        let theme = ThemeState::get();

        // Get theme-based colors if not specified
        let thumb_color = config
            .thumb_color
            .unwrap_or_else(|| theme.color(ColorToken::Border).with_alpha(0.5));
        let track_color = config
            .track_color
            .unwrap_or_else(|| theme.color(ColorToken::Surface).with_alpha(0.1));

        // Build the scroll widget with built-in scrollbar
        let mut scroll_widget = scroll()
            .direction(config.direction)
            .bounce(config.bounce)
            .scrollbar_visibility(config.visibility.to_layout())
            .scrollbar_thumb_color(thumb_color.r, thumb_color.g, thumb_color.b, thumb_color.a)
            .scrollbar_track_color(track_color.r, track_color.g, track_color.b, track_color.a);

        // Apply viewport dimensions.
        // Responsive mode follows parent container when width/height are not explicitly set.
        scroll_widget = if let Some(width) = config.width {
            scroll_widget.w(width)
        } else if config.responsive {
            scroll_widget.w_full()
        } else {
            scroll_widget.w(LEGACY_FALLBACK_WIDTH)
        };
        scroll_widget = if let Some(height) = config.height {
            scroll_widget.h(height)
        } else if config.responsive {
            scroll_widget.h_full()
        } else {
            scroll_widget.h(LEGACY_FALLBACK_HEIGHT)
        };

        // Apply scrollbar width
        if let Some(width) = config.scrollbar_width {
            scroll_widget = scroll_widget.scrollbar_width(width);
        } else {
            let scrollbar_size = if config.responsive
                && config.size == ScrollAreaSize::Medium
                && current_device_class() == DeviceClass::Mobile
            {
                ScrollbarSize::Thin
            } else {
                config.size.to_layout()
            };
            scroll_widget = scroll_widget.scrollbar_size(scrollbar_size);
        }

        // Apply corner radius
        if let Some(radius) = config.rounded {
            scroll_widget = scroll_widget.rounded(radius);
        }

        // Apply background color
        if let Some(bg) = config.bg {
            scroll_widget = scroll_widget.bg(bg);
        }

        // Add content if provided
        if let Some(content) = config.content {
            scroll_widget = scroll_widget.child(content);
        }

        Self {
            inner: scroll_widget,
        }
    }
}

/// Scroll Area component with customizable scrollbar
pub struct ScrollArea {
    inner: Scroll,
}

impl ElementBuilder for ScrollArea {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn event_handlers(&self) -> Option<&blinc_layout::event_handler::EventHandlers> {
        self.inner.event_handlers()
    }
}

/// Builder for scroll area
pub struct ScrollAreaBuilder {
    config: RefCell<ScrollAreaConfig>,
    built: OnceCell<ScrollArea>,
}

impl ScrollAreaBuilder {
    /// Create a new scroll area builder
    #[track_caller]
    pub fn new() -> Self {
        Self {
            config: RefCell::new(ScrollAreaConfig::default()),
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &ScrollArea {
        self.built.get_or_init(|| {
            // Take ownership of config, replacing with default
            let config = self.config.take();
            let built = BuiltScrollArea::from_config(config);
            ScrollArea { inner: built.inner }
        })
    }

    /// Set scrollbar visibility mode
    pub fn scrollbar(self, visibility: ScrollbarVisibility) -> Self {
        self.config.borrow_mut().visibility = visibility;
        self
    }

    /// Set scroll direction
    pub fn direction(self, direction: ScrollDirection) -> Self {
        self.config.borrow_mut().direction = direction;
        self
    }

    /// Set to vertical scrolling (default)
    pub fn vertical(self) -> Self {
        self.config.borrow_mut().direction = ScrollDirection::Vertical;
        self
    }

    /// Set to horizontal scrolling
    pub fn horizontal(self) -> Self {
        self.config.borrow_mut().direction = ScrollDirection::Horizontal;
        self
    }

    /// Set to scroll in both directions
    pub fn both_directions(self) -> Self {
        self.config.borrow_mut().direction = ScrollDirection::Both;
        self
    }

    /// Set scrollbar size preset
    pub fn size(self, size: ScrollAreaSize) -> Self {
        self.config.borrow_mut().size = size;
        self
    }

    /// Set custom scrollbar width
    pub fn scrollbar_width(self, width: f32) -> Self {
        self.config.borrow_mut().scrollbar_width = Some(width);
        self
    }

    /// Set scrollbar thumb color
    pub fn thumb_color(self, color: impl Into<Color>) -> Self {
        self.config.borrow_mut().thumb_color = Some(color.into());
        self
    }

    /// Set scrollbar track color
    pub fn track_color(self, color: impl Into<Color>) -> Self {
        self.config.borrow_mut().track_color = Some(color.into());
        self
    }

    /// Set viewport width
    pub fn w(self, width: f32) -> Self {
        self.config.borrow_mut().width = Some(width);
        self
    }

    /// Set viewport height
    pub fn h(self, height: f32) -> Self {
        self.config.borrow_mut().height = Some(height);
        self
    }

    /// Enable or disable responsive viewport defaults.
    ///
    /// - `true` (default): width/height follow parent when not explicitly set.
    /// - `false`: legacy fallback size (`300x400`) when width/height are not set.
    pub fn responsive(self, enabled: bool) -> Self {
        self.config.borrow_mut().responsive = enabled;
        self
    }

    /// Enable or disable bounce physics
    pub fn bounce(self, enabled: bool) -> Self {
        self.config.borrow_mut().bounce = enabled;
        self
    }

    /// Disable bounce physics
    pub fn no_bounce(self) -> Self {
        self.config.borrow_mut().bounce = false;
        self
    }

    /// Set corner radius
    pub fn rounded(self, radius: f32) -> Self {
        self.config.borrow_mut().rounded = Some(radius);
        self
    }

    /// Set background color
    pub fn bg(self, color: impl Into<Color>) -> Self {
        self.config.borrow_mut().bg = Some(color.into());
        self
    }

    /// Set the scrollable content
    pub fn child(self, content: Div) -> Self {
        self.config.borrow_mut().content = Some(content);
        self
    }

    /// Build the final ScrollArea component
    pub fn build_final(self) -> ScrollArea {
        let config = self.config.into_inner();
        let built = BuiltScrollArea::from_config(config);
        ScrollArea { inner: built.inner }
    }
}

impl Default for ScrollAreaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementBuilder for ScrollAreaBuilder {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        self.get_or_build().build(tree)
    }

    fn render_props(&self) -> RenderProps {
        self.get_or_build().render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.get_or_build().children_builders()
    }

    fn event_handlers(&self) -> Option<&blinc_layout::event_handler::EventHandlers> {
        self.get_or_build().event_handlers()
    }
}

/// Create a new scroll area with customizable scrollbar
///
/// # Example
///
/// ```ignore
/// use blinc_cn::prelude::*;
///
/// // Basic usage with auto-dismiss scrollbar
/// cn::scroll_area()
///     .h(400.0)
///     .child(long_content)
///
/// // Always show scrollbar
/// cn::scroll_area()
///     .scrollbar(ScrollbarVisibility::Always)
///     .h(300.0)
///     .child(content)
///
/// // Horizontal scroll
/// cn::scroll_area()
///     .horizontal()
///     .w(400.0)
///     .child(wide_content)
/// ```
#[track_caller]
pub fn scroll_area() -> ScrollAreaBuilder {
    ScrollAreaBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_core::context_state::{BlincContextState, HookState};
    use blinc_core::reactive::ReactiveGraph;
    use blinc_theme::ThemeState;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex, OnceLock};

    fn init_theme() {
        let _ = ThemeState::try_get().unwrap_or_else(|| {
            ThemeState::init_default();
            ThemeState::get()
        });
    }

    fn init_context() {
        if !BlincContextState::is_initialized() {
            let reactive = Arc::new(Mutex::new(ReactiveGraph::new()));
            let hooks = Arc::new(Mutex::new(HookState::new()));
            let dirty_flag = Arc::new(AtomicBool::new(false));
            BlincContextState::init(reactive, hooks, dirty_flag);
        }
    }

    fn context_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_scrollbar_width_presets() {
        assert_eq!(ScrollAreaSize::Small.scrollbar_width(), 4.0);
        assert_eq!(ScrollAreaSize::Medium.scrollbar_width(), 6.0);
        assert_eq!(ScrollAreaSize::Large.scrollbar_width(), 10.0);
    }

    #[test]
    fn test_scroll_area_builder_config() {
        init_theme();

        let builder = scroll_area()
            .scrollbar(ScrollbarVisibility::Always)
            .size(ScrollAreaSize::Large)
            .h(500.0)
            .w(300.0);

        let config = builder.config.borrow();
        assert_eq!(config.visibility, ScrollbarVisibility::Always);
        assert_eq!(config.size, ScrollAreaSize::Large);
        assert_eq!(config.height, Some(500.0));
        assert_eq!(config.width, Some(300.0));
    }

    #[test]
    fn test_scroll_area_is_responsive_by_default() {
        init_theme();
        let builder = scroll_area();
        let config = builder.config.borrow();
        assert!(config.responsive);
    }

    #[test]
    fn test_scroll_area_non_responsive_uses_legacy_fallback_size() {
        init_theme();

        let config = ScrollAreaConfig {
            responsive: false,
            ..Default::default()
        };
        let built = BuiltScrollArea::from_config(config);
        let physics = built.inner.physics();
        let physics = physics.lock().unwrap();

        assert_eq!(physics.viewport_width, LEGACY_FALLBACK_WIDTH);
        assert_eq!(physics.viewport_height, LEGACY_FALLBACK_HEIGHT);
    }

    #[test]
    fn test_scroll_area_explicit_size_takes_precedence_over_responsive_default() {
        init_theme();

        let config = ScrollAreaConfig {
            responsive: true,
            width: Some(320.0),
            height: Some(240.0),
            ..Default::default()
        };
        let built = BuiltScrollArea::from_config(config);
        let physics = built.inner.physics();
        let physics = physics.lock().unwrap();

        assert_eq!(physics.viewport_width, 320.0);
        assert_eq!(physics.viewport_height, 240.0);
    }

    #[test]
    fn test_scroll_area_responsive_medium_scrollbar_thins_on_mobile() {
        let _guard = context_lock().lock().unwrap();
        init_theme();
        init_context();
        BlincContextState::get().set_viewport_size(375.0, 812.0);

        let config = ScrollAreaConfig {
            responsive: true,
            size: ScrollAreaSize::Medium,
            ..Default::default()
        };
        let built = BuiltScrollArea::from_config(config);
        let physics = built.inner.physics();
        let physics = physics.lock().unwrap();

        assert_eq!(physics.config.scrollbar.size, ScrollbarSize::Thin);
    }

    #[test]
    fn test_scroll_area_responsive_medium_scrollbar_is_normal_on_desktop() {
        let _guard = context_lock().lock().unwrap();
        init_theme();
        init_context();
        BlincContextState::get().set_viewport_size(1280.0, 800.0);

        let config = ScrollAreaConfig {
            responsive: true,
            size: ScrollAreaSize::Medium,
            ..Default::default()
        };
        let built = BuiltScrollArea::from_config(config);
        let physics = built.inner.physics();
        let physics = physics.lock().unwrap();

        assert_eq!(physics.config.scrollbar.size, ScrollbarSize::Normal);
    }
}
