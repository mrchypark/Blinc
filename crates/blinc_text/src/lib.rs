//! High-quality text rendering for Blinc UI framework
//!
//! This crate provides:
//! - Font loading and parsing (TTF/OTF via ttf-parser)
//! - Text shaping (HarfBuzz via rustybuzz)
//! - Glyph rasterization
//! - Glyph atlas management
//! - Text layout engine (line breaking, alignment)

pub mod atlas;
pub mod emoji;
pub mod font;
pub mod layout;
pub mod rasterizer;
pub mod registry;
pub mod renderer;
pub mod shaper;

pub use atlas::{AtlasRegion, ColorGlyphAtlas, GlyphAtlas, GlyphInfo};
pub use emoji::{contains_emoji, is_emoji, EmojiRenderer, EmojiSprite};
pub use font::{Font, FontFace, FontMetrics, FontStyle, FontWeight};

// Re-export html-escape for entity decoding
pub use html_escape::decode_html_entities;
pub use layout::{
    LayoutOptions, LineBreakMode, PositionedGlyph, TextAlignment, TextAnchor, TextLayout,
    TextLayoutEngine,
};
pub use rasterizer::{GlyphFormat, GlyphRasterizer, RasterizedGlyph};
pub use registry::{FontRegistry, GenericFont};
pub use renderer::{ColorSpan, GlyphInstance, PreparedText, TextRenderer};
pub use shaper::{ShapedGlyph, ShapedText, TextShaper};

use thiserror::Error;

/// Text rendering errors
#[derive(Error, Debug)]
pub enum TextError {
    #[error("Failed to load font: {0}")]
    FontLoadError(String),

    #[error("Failed to parse font: {0}")]
    FontParseError(String),

    #[error("Glyph not found for codepoint: {0}")]
    GlyphNotFound(char),

    #[error("Atlas is full, cannot allocate glyph")]
    AtlasFull,

    #[error("Invalid font data")]
    InvalidFontData,
}

pub type Result<T> = std::result::Result<T, TextError>;
