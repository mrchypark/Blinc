//! Text layout engine
//!
//! Handles line breaking, text measurement, and multi-line layout.

use crate::font::FontFace;
use crate::shaper::{ShapedGlyph, ShapedText, TextShaper};

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Line break mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineBreakMode {
    /// Break at word boundaries
    #[default]
    Word,
    /// Break at character boundaries
    Character,
    /// No wrapping (single line)
    None,
}

/// Options for text layout
#[derive(Debug, Clone)]
pub struct LayoutOptions {
    /// Maximum width for line wrapping (None = no wrapping)
    pub max_width: Option<f32>,
    /// Text alignment
    pub alignment: TextAlignment,
    /// Line break mode
    pub line_break: LineBreakMode,
    /// Line height multiplier (1.0 = default)
    pub line_height: f32,
    /// Letter spacing adjustment in pixels
    pub letter_spacing: f32,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            max_width: None,
            alignment: TextAlignment::Left,
            line_break: LineBreakMode::Word,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

/// A positioned glyph ready for rendering
#[derive(Debug, Clone, Copy)]
pub struct PositionedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u16,
    /// X position in pixels
    pub x: f32,
    /// Y position in pixels (baseline)
    pub y: f32,
    /// Character this glyph represents
    pub codepoint: char,
}

/// A line of positioned glyphs
#[derive(Debug, Clone)]
pub struct LayoutLine {
    /// Glyphs in this line
    pub glyphs: Vec<PositionedGlyph>,
    /// Line width in pixels
    pub width: f32,
    /// Baseline Y position
    pub baseline_y: f32,
}

/// Result of laying out text
#[derive(Debug, Clone)]
pub struct TextLayout {
    /// Lines of positioned glyphs
    pub lines: Vec<LayoutLine>,
    /// Total width (widest line)
    pub width: f32,
    /// Total height
    pub height: f32,
}

impl TextLayout {
    /// Get all glyphs as a flat iterator
    pub fn glyphs(&self) -> impl Iterator<Item = &PositionedGlyph> {
        self.lines.iter().flat_map(|line| line.glyphs.iter())
    }

    /// Get total glyph count
    pub fn glyph_count(&self) -> usize {
        self.lines.iter().map(|l| l.glyphs.len()).sum()
    }
}

/// Text layout engine
pub struct TextLayoutEngine {
    shaper: TextShaper,
}

impl TextLayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            shaper: TextShaper::new(),
        }
    }

    /// Layout text with the given options
    pub fn layout(
        &self,
        text: &str,
        font: &FontFace,
        font_size: f32,
        options: &LayoutOptions,
    ) -> TextLayout {
        if text.is_empty() {
            return TextLayout {
                lines: Vec::new(),
                width: 0.0,
                height: 0.0,
            };
        }

        let metrics = font.metrics();
        let line_height = metrics.line_height_px(font_size) * options.line_height;
        let ascender = metrics.ascender_px(font_size);

        // Shape the entire text first
        let shaped = self.shaper.shape(text, font, font_size);

        // If no wrapping, return single line
        if options.max_width.is_none() || options.line_break == LineBreakMode::None {
            let line = self.create_line(&shaped, 0.0, ascender, options);
            let width = line.width;
            return TextLayout {
                lines: vec![line],
                width,
                height: line_height,
            };
        }

        let max_width = options.max_width.unwrap();

        // Break into lines
        let lines = self.break_lines(text, &shaped, font, font_size, max_width, options);

        // Position lines
        let mut positioned_lines = Vec::with_capacity(lines.len());
        let mut y = ascender;
        let mut max_width_found = 0.0f32;

        for line_glyphs in lines {
            let shaped_line = ShapedText {
                glyphs: line_glyphs,
                total_advance: 0, // Will be recalculated
                font_size,
                units_per_em: metrics.units_per_em,
            };

            let line = self.create_line(&shaped_line, 0.0, y, options);
            max_width_found = max_width_found.max(line.width);
            positioned_lines.push(line);
            y += line_height;
        }

        // Apply alignment
        if options.alignment != TextAlignment::Left {
            for line in &mut positioned_lines {
                let offset = match options.alignment {
                    TextAlignment::Center => (max_width - line.width) / 2.0,
                    TextAlignment::Right => max_width - line.width,
                    TextAlignment::Left => 0.0,
                };
                for glyph in &mut line.glyphs {
                    glyph.x += offset;
                }
            }
        }

        TextLayout {
            lines: positioned_lines,
            width: max_width_found,
            height: y - ascender + line_height,
        }
    }

    /// Create a layout line from shaped glyphs
    fn create_line(
        &self,
        shaped: &ShapedText,
        start_x: f32,
        baseline_y: f32,
        options: &LayoutOptions,
    ) -> LayoutLine {
        let mut glyphs = Vec::with_capacity(shaped.glyphs.len());
        let mut x = start_x;

        for glyph in &shaped.glyphs {
            let x_offset = shaped.scale(glyph.x_offset);
            let advance = shaped.scale(glyph.x_advance) + options.letter_spacing;

            glyphs.push(PositionedGlyph {
                glyph_id: glyph.glyph_id,
                x: x + x_offset,
                y: baseline_y,
                codepoint: glyph.codepoint,
            });

            x += advance;
        }

        LayoutLine {
            glyphs,
            width: x - start_x,
            baseline_y,
        }
    }

    /// Break text into lines based on max width
    fn break_lines(
        &self,
        text: &str,
        shaped: &ShapedText,
        font: &FontFace,
        font_size: f32,
        max_width: f32,
        options: &LayoutOptions,
    ) -> Vec<Vec<ShapedGlyph>> {
        let mut lines = Vec::new();
        let mut current_line: Vec<ShapedGlyph> = Vec::new();
        let mut line_width = 0.0f32;

        // Find word boundaries
        let word_breaks: Vec<usize> = text
            .char_indices()
            .filter(|(_, c)| c.is_whitespace())
            .map(|(i, _)| i)
            .collect();

        let mut last_word_end = 0;
        let mut last_word_end_glyph = 0;
        let mut last_word_width = 0.0f32;

        for (i, glyph) in shaped.glyphs.iter().enumerate() {
            let advance = shaped.scale(glyph.x_advance) + options.letter_spacing;

            // Check if this is a word boundary
            let is_word_break = word_breaks.contains(&(glyph.cluster as usize));

            if is_word_break {
                last_word_end = current_line.len();
                last_word_end_glyph = i;
                last_word_width = line_width;
            }

            // Check if we need to break
            if line_width + advance > max_width && !current_line.is_empty() {
                match options.line_break {
                    LineBreakMode::Word if last_word_end > 0 => {
                        // Break at last word boundary
                        let remaining: Vec<_> = current_line.drain(last_word_end..).collect();
                        lines.push(std::mem::take(&mut current_line));
                        current_line = remaining;
                        line_width = line_width - last_word_width;
                        last_word_end = 0;
                        last_word_width = 0.0;
                    }
                    LineBreakMode::Character | LineBreakMode::Word => {
                        // Break at current position
                        lines.push(std::mem::take(&mut current_line));
                        line_width = 0.0;
                        last_word_end = 0;
                        last_word_width = 0.0;
                    }
                    LineBreakMode::None => {
                        // No breaking (shouldn't reach here)
                    }
                }
            }

            // Skip leading whitespace on new lines
            if current_line.is_empty() && glyph.codepoint.is_whitespace() {
                continue;
            }

            current_line.push(*glyph);
            line_width += advance;
        }

        // Add remaining line
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Measure text without full layout
    pub fn measure(
        &self,
        text: &str,
        font: &FontFace,
        font_size: f32,
        options: &LayoutOptions,
    ) -> (f32, f32) {
        let layout = self.layout(text, font, font_size, options);
        (layout.width, layout.height)
    }
}

impl Default for TextLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}
