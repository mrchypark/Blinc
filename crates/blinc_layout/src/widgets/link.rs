//! Link widget for clickable text
//!
//! A styled text element that acts as a hyperlink with hover states,
//! equivalent to `<a>` in HTML. Links are underlined by default and
//! clicking opens the URL in the system browser.
//!
//! # Example
//!
//! ```ignore
//! use blinc_layout::prelude::*;
//!
//! // Default link - clicking opens URL in browser
//! link("Click here", "https://example.com")
//!
//! // Custom click handler (replaces default behavior)
//! link("Custom action", "https://example.com")
//!     .on_click(|url, ctx| println!("Custom handler: {}", url))
//!
//! // Link without underline
//! link("No underline", "https://example.com")
//!     .no_underline()
//!
//! // Link with underline only on hover
//! link("Hover to see underline", "https://example.com")
//!     .underline_on_hover()
//! ```

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use blinc_core::Color;
use blinc_theme::{ColorToken, ThemeState};

use crate::div::{div, Div, ElementBuilder};
use crate::element::RenderProps;
use crate::text::text;
use crate::tree::{LayoutNodeId, LayoutTree};

/// Open a URL in the system's default browser
///
/// This is the default action for links when clicked.
/// On platforms without the `open` crate support, this logs a warning.
pub fn open_url(url: &str) {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    {
        if let Err(e) = open::that(url) {
            tracing::warn!("Failed to open URL '{}': {}", url, e);
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        tracing::warn!("URL opening not supported on this platform: {}", url);
    }
}

/// Configuration for link styling
#[derive(Clone, Debug)]
pub struct LinkConfig {
    /// Normal text color
    pub text_color: Color,
    /// Color when hovered
    pub hover_color: Color,
    /// Font size
    pub font_size: f32,
    /// Whether to show underline always
    pub underline: bool,
    /// Whether to show underline only on hover
    pub underline_on_hover_only: bool,
}

impl Default for LinkConfig {
    fn default() -> Self {
        let theme = ThemeState::get();
        Self {
            text_color: theme.color(ColorToken::TextLink),
            hover_color: theme.color(ColorToken::PrimaryHover),
            font_size: 16.0,
            underline: true,
            underline_on_hover_only: false,
        }
    }
}

type LinkClickHandler = Arc<dyn Fn(&str, &crate::event_handler::EventContext) + Send + Sync>;

/// A hyperlink widget
pub struct Link {
    inner: Div,
    url: String,
}

impl Link {
    /// Create a new link with text and URL
    ///
    /// By default, clicking the link opens the URL in the system browser.
    /// Use `.on_click()` to override this behavior.
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        let config = LinkConfig::default();
        let label = label.into();
        let url = url.into();

        // Build the inner div with text - include all final styling here
        // so that build() and render_props()/children_builders() are consistent
        let mut text_element = text(&label)
            .size(config.font_size)
            .color(config.text_color)
            .no_cursor(); // Text inside link shouldn't override pointer cursor

        // Apply underline by default (not hover-only)
        if config.underline && !config.underline_on_hover_only {
            text_element = text_element.underline();
        }

        // Default click handler opens URL in system browser
        let url_for_click = url.clone();
        let inner = div()
            .child(text_element)
            .cursor_pointer()
            .on_click(move |_ctx| {
                open_url(&url_for_click);
            });

        Self { inner, url }
    }

    /// Set a custom click handler (receives URL and event context)
    ///
    /// This replaces the default behavior of opening the URL in the browser.
    pub fn on_click<F>(self, handler: F) -> Self
    where
        F: Fn(&str, &crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        // Rebuild without the default handler and add the custom one
        let label = self.extract_label();
        let url = self.url;
        let config = LinkConfig::default();

        let mut text_element = text(&label)
            .size(config.font_size)
            .color(config.text_color)
            .no_cursor();

        if config.underline && !config.underline_on_hover_only {
            text_element = text_element.underline();
        }

        // Use custom click handler instead of default
        let url_for_click = url.clone();
        let inner = div()
            .child(text_element)
            .cursor_pointer()
            .on_click(move |ctx| {
                handler(&url_for_click, ctx);
            });

        Self { inner, url }
    }

    /// Set the text color
    ///
    /// Note: This rebuilds the inner structure.
    pub fn text_color(self, color: Color) -> Self {
        self.rebuild_with_config(|cfg| cfg.text_color = color)
    }

    /// Set the hover color
    pub fn hover_color(self, color: Color) -> Self {
        self.rebuild_with_config(|cfg| cfg.hover_color = color)
    }

    /// Set the font size
    pub fn font_size(self, size: f32) -> Self {
        self.rebuild_with_config(|cfg| cfg.font_size = size)
    }

    /// Enable or disable underline
    pub fn underline(self, enabled: bool) -> Self {
        self.rebuild_with_config(|cfg| cfg.underline = enabled)
    }

    /// Remove underline decoration (convenience for `.underline(false)`)
    pub fn no_underline(self) -> Self {
        self.underline(false)
    }

    /// Show underline only on hover
    pub fn underline_on_hover(self) -> Self {
        self.rebuild_with_config(|cfg| {
            cfg.underline = true;
            cfg.underline_on_hover_only = true;
        })
    }

    /// Get the URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Helper to rebuild the link with a modified config
    fn rebuild_with_config<F>(self, modify: F) -> Self
    where
        F: FnOnce(&mut LinkConfig),
    {
        // Extract text content from inner (we need to rebuild anyway)
        // For simplicity, we'll extract from URL since we have it
        // Get the label from the text child if possible
        let label = self.extract_label();
        let url = self.url;

        let mut config = LinkConfig::default();
        modify(&mut config);

        let mut text_element = text(&label)
            .size(config.font_size)
            .color(config.text_color)
            .no_cursor();

        if config.underline && !config.underline_on_hover_only {
            text_element = text_element.underline();
        }

        // Default click handler opens URL in system browser
        let url_for_click = url.clone();
        let inner = div()
            .child(text_element)
            .cursor_pointer()
            .on_click(move |_ctx| {
                open_url(&url_for_click);
            });

        Self { inner, url }
    }

    /// Extract label text from the inner structure
    fn extract_label(&self) -> String {
        // The first child should be the text element
        // We can get its content via text_render_info
        if let Some(child) = self.inner.children_builders().first() {
            if let Some(info) = child.text_render_info() {
                return info.content;
            }
        }
        String::new()
    }
}

impl Deref for Link {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Link {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for Link {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        // Simply build the inner div - it already has all properties set
        self.inner.build(tree)
    }

    fn render_props(&self) -> RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn element_type_id(&self) -> crate::div::ElementTypeId {
        self.inner.element_type_id()
    }

    fn event_handlers(&self) -> Option<&crate::event_handler::EventHandlers> {
        // Delegate to inner's event_handlers implementation
        ElementBuilder::event_handlers(&self.inner)
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        self.inner.layout_style()
    }
}

/// Create a link with text and URL
///
/// # Example
///
/// ```ignore
/// link("Click here", "https://example.com")
///     .on_click(|url, _ctx| println!("Opening: {}", url))
/// ```
pub fn link(label: impl Into<String>, url: impl Into<String>) -> Link {
    Link::new(label, url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::CursorStyle;

    fn init_theme() {
        let _ = ThemeState::try_get().unwrap_or_else(|| {
            ThemeState::init_default();
            ThemeState::get()
        });
    }

    #[test]
    fn test_link_creates_element() {
        init_theme();
        let mut tree = LayoutTree::new();
        let lnk = link("Test", "https://example.com");
        lnk.build(&mut tree);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_link_stores_url() {
        init_theme();
        let lnk = link("Test", "https://example.com");
        assert_eq!(lnk.url(), "https://example.com");
    }

    #[test]
    fn test_link_has_pointer_cursor() {
        init_theme();
        let lnk = link("Test", "https://example.com");
        let props = lnk.render_props();
        assert_eq!(props.cursor, Some(CursorStyle::Pointer));
    }
}
