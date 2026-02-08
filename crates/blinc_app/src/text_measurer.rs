//! Text measurement using actual font metrics
//!
//! Provides accurate text measurement for layout by using the same font
//! as the renderer.

use blinc_layout::text_measure::{TextLayoutOptions, TextMeasurer, TextMetrics};
use blinc_layout::GenericFont as LayoutGenericFont;
use blinc_text::{
    FallbackResolver, FontFace, FontRegistry, GenericFont, LayoutOptions, TextLayoutEngine,
};
use std::sync::{Arc, Mutex};

/// Convert from layout's GenericFont to text's GenericFont
fn to_text_generic_font(layout_font: LayoutGenericFont) -> GenericFont {
    match layout_font {
        LayoutGenericFont::System => GenericFont::System,
        LayoutGenericFont::Monospace => GenericFont::Monospace,
        LayoutGenericFont::Serif => GenericFont::Serif,
        LayoutGenericFont::SansSerif => GenericFont::SansSerif,
    }
}

/// A text measurer that uses actual font metrics
///
/// This measurer uses the same font loading logic as the renderer
/// to provide accurate text dimensions for layout.
pub struct FontTextMeasurer {
    /// The font face to use for measurement (default/sans-serif)
    font: Arc<Mutex<Option<FontFace>>>,
    /// Font registry for loading different font families
    font_registry: Arc<Mutex<FontRegistry>>,
    /// The layout engine for measuring text
    layout_engine: Mutex<TextLayoutEngine>,
}

impl FontTextMeasurer {
    /// Create a new font text measurer.
    ///
    /// Uses the global shared font registry to minimize memory usage.
    /// Apple Color Emoji alone is 180MB - sharing prevents loading it multiple times.
    pub fn new() -> Self {
        let mut measurer = Self {
            font: Arc::new(Mutex::new(None)),
            font_registry: blinc_text::global_font_registry(),
            layout_engine: Mutex::new(TextLayoutEngine::new()),
        };
        measurer.load_system_font();
        measurer
    }

    /// Create a font text measurer with a shared font registry
    ///
    /// Use this to share the font registry with the text renderer,
    /// ensuring consistent font loading and metrics between measurement
    /// and rendering.
    pub fn with_shared_registry(font_registry: Arc<Mutex<FontRegistry>>) -> Self {
        let measurer = Self {
            font: Arc::new(Mutex::new(None)),
            font_registry,
            layout_engine: Mutex::new(TextLayoutEngine::new()),
        };
        // Note: system font loading is skipped since the registry is shared
        // and should already be initialized by the renderer
        measurer
    }

    /// Load the system default font
    fn load_system_font(&mut self) {
        for font_path in crate::system_font_paths() {
            let path = std::path::Path::new(font_path);
            if path.exists() {
                if let Ok(data) = std::fs::read(path) {
                    if let Ok(font) = FontFace::from_data(data) {
                        *self.font.lock().unwrap() = Some(font);
                        break;
                    }
                }
            }
        }
    }

    /// Load a custom font from data
    pub fn load_font_data(&self, data: Vec<u8>) -> Result<(), blinc_text::TextError> {
        let font = FontFace::from_data(data)?;
        *self.font.lock().unwrap() = Some(font);
        Ok(())
    }

    /// Fallback estimation when no font is loaded
    fn estimate_size(text: &str, font_size: f32, options: &TextLayoutOptions) -> TextMetrics {
        let char_count = text.chars().count() as f32;
        let word_count = text.split_whitespace().count().max(1) as f32;

        // Base width: ~0.55 * font_size per character
        let base_char_width = font_size * 0.55;
        let base_width = char_count * base_char_width;

        // Add letter spacing
        let letter_spacing_total = if char_count > 1.0 {
            (char_count - 1.0) * options.letter_spacing
        } else {
            0.0
        };

        // Add word spacing
        let word_spacing_total = if word_count > 1.0 {
            (word_count - 1.0) * options.word_spacing
        } else {
            0.0
        };

        let total_width = base_width + letter_spacing_total + word_spacing_total;

        // Handle wrapping
        let (width, line_count) = if let Some(max_width) = options.max_width {
            if total_width > max_width && max_width > 0.0 {
                let lines = (total_width / max_width).ceil() as u32;
                (max_width, lines.max(1))
            } else {
                (total_width, 1)
            }
        } else {
            (total_width, 1)
        };

        let line_height_px = font_size * options.line_height;
        let height = line_height_px * line_count as f32;

        TextMetrics {
            width,
            height,
            ascender: font_size * 0.8,
            descender: font_size * -0.2,
            line_count,
        }
    }
}

impl Default for FontTextMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextMeasurer for FontTextMeasurer {
    fn measure_with_options(
        &self,
        text: &str,
        font_size: f32,
        options: &TextLayoutOptions,
    ) -> TextMetrics {
        // Determine which font to use based on options
        let generic_font = to_text_generic_font(options.generic_font);

        // Fast path: use cached fonts only (never load during measurement)
        // Use weight and italic from options to get the correct font variant
        let registry = self.font_registry.lock().unwrap();
        let font = match registry.get_for_render_with_style(
            options.font_name.as_deref(),
            generic_font,
            options.font_weight,
            options.italic,
        ) {
            Some(f) => f,
            None => return Self::estimate_size(text, font_size, options),
        };
        drop(registry); // Release lock before layout

        // Convert our options to blinc_text options
        let mut layout_opts = LayoutOptions::default();
        layout_opts.line_height = options.line_height;
        layout_opts.letter_spacing = options.letter_spacing;
        if let Some(max_width) = options.max_width {
            layout_opts.max_width = Some(max_width);
        } else {
            // No wrapping for single-line measurement
            layout_opts.line_break = blinc_text::LineBreakMode::None;
        }

        let layout_engine = self.layout_engine.lock().unwrap();
        let layout = layout_engine.layout(text, &font, font_size, &layout_opts);

        // Width correction for font fallback:
        // Layout positions/width are based on the primary font only. If the primary font is missing
        // glyphs (common for CJK/Arabic/etc), rendering will fallback to system fonts which often
        // have different advance widths. We apply the same "x_offset" correction logic as the
        // renderer so measurement matches what will be drawn.
        let mut corrected_width: f32 = 0.0;
        let mut x_offset: f32;

        let mut resolver = FallbackResolver::new(options.font_weight, options.italic);

        for line in &layout.lines {
            x_offset = 0.0;

            for (i, glyph) in line.glyphs.iter().enumerate() {
                if glyph.codepoint.is_whitespace() {
                    continue;
                }
                if blinc_text::emoji::is_variation_selector(glyph.codepoint)
                    || blinc_text::emoji::is_zwj(glyph.codepoint)
                {
                    continue;
                }

                // Suppress duplicate emoji glyphs produced by HarfBuzz cluster mapping.
                if i > 0
                    && blinc_text::emoji::should_skip_duplicate_emoji(&line.glyphs[i - 1], glyph)
                {
                    continue;
                }

                let is_emoji_char = blinc_text::is_emoji(glyph.codepoint);
                let primary_has_glyph = glyph.glyph_id != 0 && font.has_glyph(glyph.codepoint);
                let needs_fallback = !primary_has_glyph || is_emoji_char;

                if !needs_fallback {
                    continue;
                }

                let candidates = resolver.candidates_for_char(
                    self.font_registry.as_ref(),
                    glyph.codepoint,
                    is_emoji_char,
                );
                let fallback = candidates.into_iter().find_map(|c| {
                    let gid = c.face.glyph_id(glyph.codepoint)?;
                    if gid == 0 {
                        None
                    } else {
                        Some((c.face, gid))
                    }
                });
                let Some((fallback_font, fallback_gid)) = fallback else {
                    continue;
                };

                // Compute fallback advance in pixels (include letter spacing to match layout)
                let adv_units = fallback_font.glyph_advance(fallback_gid).unwrap_or(0) as f32;
                let scale = font_size / fallback_font.metrics().units_per_em as f32;
                let fallback_advance = adv_units * scale + layout_opts.letter_spacing;

                // Primary advance as implied by the primary layout (distance to next glyph)
                let primary_advance = if i + 1 < line.glyphs.len() {
                    (line.glyphs[i + 1].x - glyph.x).max(0.0)
                } else {
                    fallback_advance
                };

                x_offset += fallback_advance - primary_advance;
            }

            corrected_width = corrected_width.max((line.width + x_offset).max(0.0));
        }

        // Get font metrics
        let metrics = font.metrics();
        let ascender = metrics.ascender_px(font_size);
        let descender = metrics.descender_px(font_size);

        TextMetrics {
            width: corrected_width.max(layout.width),
            height: layout.height,
            ascender,
            descender,
            line_count: layout.lines.len() as u32,
        }
    }
}

/// Initialize the global text measurer with font support
///
/// Call this at application startup to enable accurate text measurement.
/// This should be called before any UI elements are created.
///
/// Note: For optimal text rendering, use `init_text_measurer_with_registry`
/// to share the font registry with the text renderer.
pub fn init_text_measurer() {
    let measurer = Arc::new(FontTextMeasurer::new());
    blinc_layout::set_text_measurer(measurer);
}

/// Initialize the global text measurer with a shared font registry
///
/// This ensures the text measurer uses the same fonts as the renderer,
/// providing accurate text measurement that matches rendered text exactly.
///
/// Call this after creating the BlincApp/TextRenderingContext:
///
/// ```ignore
/// let (app, surface) = BlincApp::with_window(window, None)?;
/// init_text_measurer_with_registry(app.font_registry());
/// ```
pub fn init_text_measurer_with_registry(font_registry: Arc<Mutex<FontRegistry>>) {
    let measurer = Arc::new(FontTextMeasurer::with_shared_registry(font_registry));
    blinc_layout::set_text_measurer(measurer);
}
