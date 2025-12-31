//! Link widget for clickable text
//!
//! A styled text element that acts as a hyperlink with hover states,
//! equivalent to `<a>` in HTML. Links are underlined by default.
//!
//! # Example
//!
//! ```ignore
//! use blinc_layout::prelude::*;
//!
//! // Default link with underline
//! link("Click here", "https://example.com")
//!     .on_click(|url, ctx| open_url(url))
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
    label: String,
    config: LinkConfig,
    click_handler: Option<LinkClickHandler>,
}

impl Link {
    /// Create a new link with text and URL
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        let config = LinkConfig::default();
        let label = label.into();
        let url = url.into();

        // Build the inner div with text
        let text_element = text(&label)
            .size(config.font_size)
            .color(config.text_color);

        let inner = div().child(text_element);

        Self {
            inner,
            url,
            label,
            config,
            click_handler: None,
        }
    }

    /// Set the click handler (receives URL and event context)
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.click_handler = Some(Arc::new(handler));
        self
    }

    /// Set the text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.config.text_color = color;
        self
    }

    /// Set the hover color
    pub fn hover_color(mut self, color: Color) -> Self {
        self.config.hover_color = color;
        self
    }

    /// Set the font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.config.font_size = size;
        self
    }

    /// Enable or disable underline
    pub fn underline(mut self, enabled: bool) -> Self {
        self.config.underline = enabled;
        self
    }

    /// Remove underline decoration (convenience for `.underline(false)`)
    pub fn no_underline(mut self) -> Self {
        self.config.underline = false;
        self
    }

    /// Show underline only on hover
    pub fn underline_on_hover(mut self) -> Self {
        self.config.underline = true;
        self.config.underline_on_hover_only = true;
        self
    }

    /// Get the URL
    pub fn url(&self) -> &str {
        &self.url
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
        // Build with text styling
        // Note: Hover state visual changes would need Stateful for efficient updates
        let mut text_element = text(&self.label)
            .size(self.config.font_size)
            .color(self.config.text_color);

        // Apply underline if enabled (and not hover-only mode)
        if self.config.underline && !self.config.underline_on_hover_only {
            text_element = text_element.underline();
        }

        let mut inner = div().child(text_element);

        // Add click handler
        if let Some(handler) = &self.click_handler {
            let handler = Arc::clone(handler);
            let url = self.url.clone();
            inner = inner.on_click(move |ctx| {
                handler(&url, ctx);
            });
        }

        inner.build(tree)
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
        assert!(tree.len() > 0);
    }

    #[test]
    fn test_link_stores_url() {
        init_theme();
        let lnk = link("Test", "https://example.com");
        assert_eq!(lnk.url(), "https://example.com");
    }
}
