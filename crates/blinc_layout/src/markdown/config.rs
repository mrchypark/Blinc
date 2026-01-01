//! Configuration for markdown rendering

use blinc_core::Color;
use blinc_theme::{ColorToken, ThemeState};

/// Configuration for markdown rendering
#[derive(Clone, Debug)]
pub struct MarkdownConfig {
    // Typography sizes
    /// H1 heading size
    pub h1_size: f32,
    /// H2 heading size
    pub h2_size: f32,
    /// H3 heading size
    pub h3_size: f32,
    /// H4 heading size
    pub h4_size: f32,
    /// H5 heading size
    pub h5_size: f32,
    /// H6 heading size
    pub h6_size: f32,
    /// Body text size
    pub body_size: f32,
    /// Code text size
    pub code_size: f32,

    // Colors
    /// Primary text color
    pub text_color: Color,
    /// Secondary text color (for muted content)
    pub text_secondary: Color,
    /// Link text color
    pub link_color: Color,
    /// Code background color
    pub code_bg: Color,
    /// Code text color
    pub code_text: Color,
    /// Blockquote border color
    pub blockquote_border: Color,
    /// Blockquote background color
    pub blockquote_bg: Color,
    /// Horizontal rule color
    pub hr_color: Color,

    // Spacing
    /// Spacing between paragraphs
    pub paragraph_spacing: f32,
    /// Spacing after headings
    pub heading_spacing: f32,
    /// List item indent
    pub list_indent: f32,
    /// List item spacing
    pub list_item_spacing: f32,
    /// Blockquote padding
    pub blockquote_padding: f32,
    /// Code block padding
    pub code_padding: f32,
    /// List marker width (space reserved for bullet/number)
    pub list_marker_width: f32,
    /// List marker gap (space between marker and content)
    pub list_marker_gap: f32,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        let theme = ThemeState::get();
        Self {
            // Typography
            h1_size: 32.0,
            h2_size: 28.0,
            h3_size: 24.0,
            h4_size: 20.0,
            h5_size: 18.0,
            h6_size: 16.0,
            body_size: 16.0,
            code_size: 14.0,

            // Colors from theme
            text_color: theme.color(ColorToken::TextPrimary),
            text_secondary: theme.color(ColorToken::TextSecondary),
            link_color: theme.color(ColorToken::TextLink),
            code_bg: theme.color(ColorToken::SurfaceOverlay),
            code_text: theme.color(ColorToken::TextPrimary),
            blockquote_border: theme.color(ColorToken::Border),
            blockquote_bg: theme.color(ColorToken::SurfaceOverlay),
            hr_color: theme.color(ColorToken::Border),

            // Spacing
            paragraph_spacing: 16.0,
            heading_spacing: 24.0,
            list_indent: 0.0,
            list_item_spacing: 4.0,
            blockquote_padding: 16.0,
            code_padding: 12.0,
            list_marker_width: 12.0,
            list_marker_gap: 4.0,
        }
    }
}

impl MarkdownConfig {
    /// Create a new config with theme defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a light theme config suitable for white backgrounds
    /// Uses the global ThemeState for colors
    pub fn light() -> Self {
        let theme = ThemeState::get();
        Self {
            // Typography - slightly smaller for tighter layout
            h1_size: 24.0,
            h2_size: 20.0,
            h3_size: 17.0,
            h4_size: 15.0,
            h5_size: 14.0,
            h6_size: 13.0,
            body_size: 14.0,
            code_size: 13.0,

            // Colors from global theme
            text_color: theme.color(ColorToken::TextPrimary),
            text_secondary: theme.color(ColorToken::TextSecondary),
            link_color: theme.color(ColorToken::TextLink),
            code_bg: theme.color(ColorToken::SurfaceOverlay),
            code_text: theme.color(ColorToken::TextPrimary),
            blockquote_border: theme.color(ColorToken::Border),
            blockquote_bg: theme.color(ColorToken::SurfaceOverlay),
            hr_color: theme.color(ColorToken::Border),

            // Tight spacing
            paragraph_spacing: 6.0,
            heading_spacing: 8.0,
            list_indent: 0.0,
            list_item_spacing: 4.0,
            blockquote_padding: 8.0,
            code_padding: 8.0,
            list_marker_width: 12.0,
            list_marker_gap: 4.0,
        }
    }

    /// Set the body text size
    pub fn body_size(mut self, size: f32) -> Self {
        self.body_size = size;
        self
    }

    /// Set the link color
    pub fn link_color(mut self, color: Color) -> Self {
        self.link_color = color;
        self
    }

    /// Set the text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set paragraph spacing
    pub fn paragraph_spacing(mut self, spacing: f32) -> Self {
        self.paragraph_spacing = spacing;
        self
    }

    /// Set blockquote background color
    pub fn blockquote_bg(mut self, color: Color) -> Self {
        self.blockquote_bg = color;
        self
    }

    /// Set blockquote border color
    pub fn blockquote_border(mut self, color: Color) -> Self {
        self.blockquote_border = color;
        self
    }
}
