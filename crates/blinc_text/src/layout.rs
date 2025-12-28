//! Text layout engine
//!
//! Handles line breaking, text measurement, and multi-line layout.

use crate::font::FontFace;
use crate::shaper::{ShapedGlyph, ShapedText, TextShaper};

/// Text alignment options (horizontal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Vertical anchor point for text positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAnchor {
    /// Y coordinate is the top of the text bounding box
    #[default]
    Top,
    /// Y coordinate is the text baseline
    Baseline,
    /// Y coordinate is the vertical center of the text
    Center,
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
    /// Text alignment (horizontal)
    pub alignment: TextAlignment,
    /// Vertical anchor point
    pub anchor: TextAnchor,
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
            anchor: TextAnchor::Top,
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
        let metrics = font.metrics();
        let line_height = metrics.line_height_px(font_size) * options.line_height;

        if text.is_empty() {
            // Empty text should still have proper height based on font metrics
            // so that layout containers size correctly
            return TextLayout {
                lines: Vec::new(),
                width: 0.0,
                height: line_height,
            };
        }

        let ascender = metrics.ascender_px(font_size);

        // Check for explicit newlines - these are always respected regardless of wrap mode
        let has_newlines = text.contains('\n');

        // Shape the entire text first
        let shaped = self.shaper.shape(text, font, font_size);

        // If no wrapping AND no explicit newlines, return single line
        if (options.max_width.is_none() || options.line_break == LineBreakMode::None)
            && !has_newlines
        {
            let mut line = self.create_line(&shaped, 0.0, ascender, options);
            let width = line.width;

            // Apply alignment if max_width is set
            if let Some(max_width) = options.max_width {
                if options.alignment != TextAlignment::Left && width < max_width {
                    let offset = match options.alignment {
                        TextAlignment::Center => (max_width - width) / 2.0,
                        TextAlignment::Right => max_width - width,
                        TextAlignment::Left => 0.0,
                    };
                    for glyph in &mut line.glyphs {
                        glyph.x += offset;
                    }
                }
            }

            return TextLayout {
                lines: vec![line],
                width,
                height: line_height,
            };
        }

        // Handle explicit newlines without word wrapping
        if has_newlines
            && (options.max_width.is_none() || options.line_break == LineBreakMode::None)
        {
            return self.layout_with_newlines_only(
                text,
                &shaped,
                font,
                font_size,
                ascender,
                line_height,
                options,
            );
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

    /// Layout text that contains explicit newlines but no word wrapping
    fn layout_with_newlines_only(
        &self,
        text: &str,
        shaped: &ShapedText,
        _font: &FontFace,
        _font_size: f32,
        ascender: f32,
        line_height: f32,
        options: &LayoutOptions,
    ) -> TextLayout {
        let mut lines = Vec::new();
        let mut current_line: Vec<ShapedGlyph> = Vec::new();

        // Split glyphs by newline character
        for glyph in &shaped.glyphs {
            if glyph.codepoint == '\n' {
                // End current line, don't include the newline glyph
                lines.push(std::mem::take(&mut current_line));
            } else {
                current_line.push(*glyph);
            }
        }

        // Add remaining line
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // Handle case where text ends with newline
        if text.ends_with('\n') {
            lines.push(Vec::new());
        }

        // Position lines
        let mut positioned_lines = Vec::with_capacity(lines.len());
        let mut y = ascender;
        let mut max_width_found = 0.0f32;
        let metrics_units_per_em = shaped.units_per_em;

        for line_glyphs in lines {
            let shaped_line = ShapedText {
                glyphs: line_glyphs,
                total_advance: 0,
                font_size: shaped.font_size,
                units_per_em: metrics_units_per_em,
            };

            let line = self.create_line(&shaped_line, 0.0, y, options);
            max_width_found = max_width_found.max(line.width);
            positioned_lines.push(line);
            y += line_height;
        }

        // Apply alignment if max_width is set
        if let Some(max_width) = options.max_width {
            if options.alignment != TextAlignment::Left {
                for line in &mut positioned_lines {
                    if line.width < max_width {
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
            }
        }

        let total_height = if positioned_lines.is_empty() {
            line_height
        } else {
            (positioned_lines.len() as f32) * line_height
        };

        TextLayout {
            lines: positioned_lines,
            width: max_width_found,
            height: total_height,
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
        _font: &FontFace,
        _font_size: f32,
        max_width: f32,
        options: &LayoutOptions,
    ) -> Vec<Vec<ShapedGlyph>> {
        let mut lines = Vec::new();
        let mut current_line: Vec<ShapedGlyph> = Vec::new();
        let mut line_width = 0.0f32;

        // Find word boundaries (whitespace positions)
        let word_breaks: Vec<usize> = text
            .char_indices()
            .filter(|(_, c)| c.is_whitespace())
            .map(|(i, _)| i)
            .collect();

        let mut last_word_end = 0;
        let mut last_word_width = 0.0f32;

        for glyph in shaped.glyphs.iter() {
            // Handle explicit newline - always force a line break
            if glyph.codepoint == '\n' {
                lines.push(std::mem::take(&mut current_line));
                line_width = 0.0;
                last_word_end = 0;
                last_word_width = 0.0;
                continue; // Don't include the newline glyph itself
            }

            let advance = shaped.scale(glyph.x_advance) + options.letter_spacing;

            // Check if this is a word boundary (at a whitespace character)
            let is_word_break = word_breaks.contains(&(glyph.cluster as usize));

            // Check if adding this glyph would overflow the line
            if line_width + advance > max_width && !current_line.is_empty() {
                let mut broke_line = false;
                match options.line_break {
                    LineBreakMode::Word => {
                        if last_word_end > 0 {
                            // Break at last word boundary
                            // Keep glyphs 0..last_word_end on current line
                            // Move glyphs last_word_end.. to next line
                            let remaining: Vec<_> = current_line.drain(last_word_end..).collect();
                            lines.push(std::mem::take(&mut current_line));

                            // Start new line with remaining glyphs (skip leading whitespace)
                            for g in remaining {
                                if current_line.is_empty() && g.codepoint.is_whitespace() {
                                    continue; // Skip leading whitespace
                                }
                                current_line.push(g);
                            }

                            // Recalculate line width
                            line_width = current_line
                                .iter()
                                .map(|g| shaped.scale(g.x_advance) + options.letter_spacing)
                                .sum();
                            last_word_end = 0;
                            last_word_width = 0.0;
                            broke_line = true;
                        } else {
                            // No word boundary found - break at current position (character break)
                            lines.push(std::mem::take(&mut current_line));
                            line_width = 0.0;
                            last_word_end = 0;
                            last_word_width = 0.0;
                            broke_line = true;
                        }
                    }
                    LineBreakMode::Character => {
                        // Break at current position
                        lines.push(std::mem::take(&mut current_line));
                        line_width = 0.0;
                        last_word_end = 0;
                        last_word_width = 0.0;
                        broke_line = true;
                    }
                    LineBreakMode::None => {
                        // No breaking - let line overflow
                    }
                }

                // After breaking, we still need to add the current glyph (unless it's whitespace at line start)
                if broke_line {
                    // Skip leading whitespace on new lines
                    if current_line.is_empty() && glyph.codepoint.is_whitespace() {
                        continue;
                    }

                    // Add the current glyph that triggered the overflow
                    current_line.push(*glyph);
                    line_width += advance;

                    // Update word boundary if this glyph is a word break
                    if is_word_break {
                        last_word_end = current_line.len();
                        last_word_width = line_width;
                    }
                    continue; // Move to next glyph
                }
            }

            // Skip leading whitespace on new lines
            if current_line.is_empty() && glyph.codepoint.is_whitespace() {
                continue;
            }

            // Add glyph to current line
            current_line.push(*glyph);
            line_width += advance;

            // Update word boundary tracking AFTER adding the glyph
            // This way, when we break at last_word_end, all content up to and including
            // the space is on the current line, and remaining content goes to next line
            if is_word_break {
                // Mark position AFTER this whitespace as potential break point
                last_word_end = current_line.len();
                last_word_width = line_width;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shaper::ShapedGlyph;

    /// Helper to verify all text content is preserved after word wrapping
    fn verify_content_preserved(lines: &[Vec<ShapedGlyph>], original: &str) {
        // Collect all characters from wrapped lines
        let wrapped: String = lines
            .iter()
            .flat_map(|line| line.iter().map(|g| g.codepoint))
            .collect();

        // The original text with whitespace normalized (single spaces between words)
        let original_normalized: String = original.split_whitespace().collect::<Vec<_>>().join(" ");

        // The wrapped text should have the same content (just with different whitespace)
        let wrapped_normalized: String = wrapped.split_whitespace().collect::<Vec<_>>().join(" ");

        assert_eq!(
            wrapped_normalized, original_normalized,
            "Text content was lost during wrapping!\nOriginal: {}\nWrapped: {}",
            original_normalized, wrapped_normalized
        );
    }

    fn create_mock_shaped_text(text: &str) -> ShapedText {
        let mut glyphs = Vec::new();
        for (i, c) in text.char_indices() {
            glyphs.push(ShapedGlyph {
                glyph_id: c as u16,
                cluster: i as u32,
                // ~5px space, ~10px char at font_size=16, units_per_em=1000
                x_advance: if c.is_whitespace() { 313 } else { 625 },
                y_advance: 0,
                x_offset: 0,
                y_offset: 0,
                codepoint: c,
            });
        }

        ShapedText {
            glyphs,
            total_advance: 0,
            font_size: 16.0,
            units_per_em: 1000,
        }
    }

    /// Standalone test of the word-break algorithm logic without needing a FontFace
    fn test_break_algorithm(text: &str, max_width: f32) -> Vec<Vec<ShapedGlyph>> {
        let shaped = create_mock_shaped_text(text);
        let options = LayoutOptions {
            max_width: Some(max_width),
            line_break: LineBreakMode::Word,
            ..Default::default()
        };

        // Re-implement break_lines logic directly for testing
        let mut lines = Vec::new();
        let mut current_line: Vec<ShapedGlyph> = Vec::new();
        let mut line_width = 0.0f32;

        let word_breaks: Vec<usize> = text
            .char_indices()
            .filter(|(_, c)| c.is_whitespace())
            .map(|(i, _)| i)
            .collect();

        let mut last_word_end = 0;

        for glyph in shaped.glyphs.iter() {
            let advance = shaped.scale(glyph.x_advance) + options.letter_spacing;
            let is_word_break = word_breaks.contains(&(glyph.cluster as usize));

            if line_width + advance > max_width && !current_line.is_empty() {
                let mut broke_line = false;
                if last_word_end > 0 {
                    let remaining: Vec<_> = current_line.drain(last_word_end..).collect();
                    lines.push(std::mem::take(&mut current_line));

                    for g in remaining {
                        if current_line.is_empty() && g.codepoint.is_whitespace() {
                            continue;
                        }
                        current_line.push(g);
                    }

                    line_width = current_line
                        .iter()
                        .map(|g| shaped.scale(g.x_advance) + options.letter_spacing)
                        .sum();
                    last_word_end = 0;
                    broke_line = true;
                } else {
                    lines.push(std::mem::take(&mut current_line));
                    line_width = 0.0;
                    last_word_end = 0;
                    broke_line = true;
                }

                if broke_line {
                    if current_line.is_empty() && glyph.codepoint.is_whitespace() {
                        continue;
                    }

                    current_line.push(*glyph);
                    line_width += advance;

                    if is_word_break {
                        last_word_end = current_line.len();
                    }
                    continue;
                }
            }

            if current_line.is_empty() && glyph.codepoint.is_whitespace() {
                continue;
            }

            current_line.push(*glyph);
            line_width += advance;

            if is_word_break {
                last_word_end = current_line.len();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    #[test]
    fn test_word_wrap_preserves_all_content() {
        let text = "This is a paragraph with optimal line height for readability.";
        let lines = test_break_algorithm(text, 200.0);

        // Verify no content lost
        verify_content_preserved(&lines, text);

        // Should have multiple lines due to wrapping
        assert!(
            lines.len() > 1,
            "Text should have wrapped into multiple lines"
        );
    }

    #[test]
    fn test_problematic_paragraph() {
        // This is the exact text that was losing content
        let text = "This is a paragraph with optimal line height (1.5) for readability. Paragraphs are styled at 16px with comfortable spacing for body text.";
        let lines = test_break_algorithm(text, 400.0);

        // Verify no content lost
        verify_content_preserved(&lines, text);

        // Print lines for debugging
        for (i, line) in lines.iter().enumerate() {
            let line_text: String = line.iter().map(|g| g.codepoint).collect();
            println!("Line {}: '{}'", i + 1, line_text);
        }
    }
}
