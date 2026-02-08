//! Utilities for "system fallback" font selection.
//!
//! The goal is to avoid repeated full fontdb scans for scripts that have lots of
//! distinct codepoints (Hangul/CJK/etc). We group codepoints into "script-ish"
//! buckets for caching, while still validating that the chosen face covers the
//! exact character at use sites.

use crate::emoji::{is_emoji, is_variation_selector, is_zwj, should_skip_duplicate_emoji};
use crate::font::FontFace;
use crate::layout::{PositionedGlyph, TextLayout};
use crate::registry::{FontRegistry, GenericFont};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

const BUCKET_HANGUL: u32 = 0x11_0000;
const BUCKET_KANA: u32 = 0x11_0001;
const BUCKET_HAN: u32 = 0x11_0002;
const BUCKET_ARABIC: u32 = 0x11_0003;
const BUCKET_DEVANAGARI: u32 = 0x11_0004;
const BUCKET_THAI: u32 = 0x11_0005;
const BUCKET_HEBREW: u32 = 0x11_0006;
const BUCKET_CYRILLIC: u32 = 0x11_0007;
const BUCKET_GREEK: u32 = 0x11_0008;

/// Shared helper for applying per-line fallback width correction.
///
/// Layout positions/width are computed from the primary font. When we render some glyphs using a
/// fallback font, advances can differ; this helper tracks the running `x_offset` and the maximum
/// corrected line width.
pub struct WidthCorrector {
    x_offset: f32,
    corrected_width: f32,
}

impl WidthCorrector {
    pub fn new() -> Self {
        Self {
            x_offset: 0.0,
            corrected_width: 0.0,
        }
    }

    pub fn begin_line(&mut self) {
        self.x_offset = 0.0;
    }

    pub fn x_offset(&self) -> f32 {
        self.x_offset
    }

    pub fn apply_advance(&mut self, primary_advance: f32, fallback_advance: f32) {
        self.x_offset += fallback_advance - primary_advance;
    }

    pub fn end_line(&mut self, line_width: f32) {
        self.corrected_width = self
            .corrected_width
            .max((line_width + self.x_offset).max(0.0));
    }

    pub fn corrected_width(&self) -> f32 {
        self.corrected_width
    }
}

pub trait FallbackWalkHandler {
    type Error;

    fn on_skip(&mut self) -> std::result::Result<(), Self::Error>;
    fn on_primary(&mut self, glyph: PositionedGlyph) -> std::result::Result<(), Self::Error>;

    /// Return `Ok(Some(fallback_advance_px))` to accept this candidate and apply width correction.
    /// Return `Ok(None)` to reject this candidate and try the next one.
    fn on_fallback(
        &mut self,
        glyph: PositionedGlyph,
        candidate: &FallbackCandidate,
    ) -> std::result::Result<Option<f32>, Self::Error>;
}

/// Walk a laid out text run and apply fallback + width correction in one place.
///
/// This centralizes the common loop used by both the renderer (rasterization) and the measurer
/// (metrics-only), to reduce drift over time.
pub fn walk_layout_with_fallback<H: FallbackWalkHandler>(
    layout: &TextLayout,
    primary_font: &FontFace,
    registry: &Mutex<FontRegistry>,
    weight: u16,
    italic: bool,
    handler: &mut H,
) -> std::result::Result<f32, H::Error> {
    let mut resolver = FallbackResolver::new(weight, italic);
    let mut width_corrector = WidthCorrector::new();

    for line in &layout.lines {
        width_corrector.begin_line();

        for (i, positioned) in line.glyphs.iter().enumerate() {
            if positioned.codepoint.is_whitespace() {
                handler.on_skip()?;
                continue;
            }
            if is_variation_selector(positioned.codepoint) || is_zwj(positioned.codepoint) {
                handler.on_skip()?;
                continue;
            }
            if i > 0 && should_skip_duplicate_emoji(&line.glyphs[i - 1], positioned) {
                handler.on_skip()?;
                continue;
            }

            let is_emoji_char = is_emoji(positioned.codepoint);
            let primary_has_glyph =
                positioned.glyph_id != 0 && primary_font.has_glyph(positioned.codepoint);
            let needs_fallback = !primary_has_glyph || is_emoji_char;

            let mut adjusted = *positioned;
            adjusted.x += width_corrector.x_offset();

            if needs_fallback {
                let candidates =
                    resolver.candidates_for_char(registry, positioned.codepoint, is_emoji_char);

                let mut handled = false;
                for candidate in &candidates {
                    if let Some(fallback_advance) = handler.on_fallback(adjusted, candidate)? {
                        let primary_advance = if i + 1 < line.glyphs.len() {
                            (line.glyphs[i + 1].x - positioned.x).max(0.0)
                        } else {
                            // For the last glyph, infer the primary advance from the line width.
                            // This preserves width correction when the last glyph is rendered via
                            // fallback and the fallback advance differs from the primary font.
                            (line.width - positioned.x).max(0.0)
                        };
                        width_corrector.apply_advance(primary_advance, fallback_advance);
                        handled = true;
                        break;
                    }
                }
                if handled {
                    continue;
                }
            }

            handler.on_primary(adjusted)?;
        }

        width_corrector.end_line(line.width);
    }

    Ok(width_corrector.corrected_width())
}

/// Returns `true` for scripts where using cmap glyph id alone is likely incorrect
/// (GSUB/GPOS shaping required).
///
/// This helper exists so the renderer and the measurer stay consistent about when
/// to invoke HarfBuzz for fallback glyph resolution.
pub fn needs_single_char_shaping(c: char) -> bool {
    matches!(
        fallback_bucket_key(c),
        BUCKET_ARABIC | BUCKET_DEVANAGARI | BUCKET_THAI | BUCKET_HEBREW
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackKind {
    Emoji,
    Symbol,
    System,
}

#[derive(Debug, Clone)]
pub struct FallbackCandidate {
    pub face: Arc<FontFace>,
    pub kind: FallbackKind,
    pub use_color: bool,
}

/// Per-call fallback resolver that caches system fallback choices by bucket and lazily
/// loads emoji/symbol faces only when needed.
///
/// This is used by both renderer and measurer so that fallback selection stays consistent.
pub struct FallbackResolver {
    weight: u16,
    italic: bool,

    emoji_font: Option<Arc<FontFace>>,
    symbol_font: Option<Arc<FontFace>>,
    emoji_loaded: bool,
    symbol_loaded: bool,

    sys_cache: FxHashMap<u32, Option<Arc<FontFace>>>,
}

impl FallbackResolver {
    pub fn new(weight: u16, italic: bool) -> Self {
        Self {
            weight,
            italic,
            emoji_font: None,
            symbol_font: None,
            emoji_loaded: false,
            symbol_loaded: false,
            sys_cache: FxHashMap::default(),
        }
    }

    fn ensure_symbol_loaded(&mut self, registry: &Mutex<FontRegistry>) {
        if self.symbol_loaded {
            return;
        }
        let mut reg = registry.lock().unwrap();
        self.symbol_font = reg.load_generic(GenericFont::Symbol).ok();
        self.symbol_loaded = true;
    }

    fn ensure_emoji_loaded(&mut self, registry: &Mutex<FontRegistry>) {
        if self.emoji_loaded {
            return;
        }
        let mut reg = registry.lock().unwrap();
        self.emoji_font = reg.load_generic(GenericFont::Emoji).ok();
        self.emoji_loaded = true;
    }

    fn system_fallback_for_char(
        &mut self,
        registry: &Mutex<FontRegistry>,
        c: char,
    ) -> Option<Arc<FontFace>> {
        let bucket = fallback_bucket_key(c);
        if let Some(Some(face)) = self.sys_cache.get(&bucket) {
            if face.has_glyph(c) {
                return Some(Arc::clone(face));
            }
        }

        // Not in cache or cached face doesn't cover this char: resolve and update.
        let new_face = {
            let mut reg = registry.lock().unwrap();
            reg.load_fallback_for_char(c, self.weight, self.italic)
        };
        self.sys_cache.insert(bucket, new_face.clone());
        new_face
    }

    pub fn candidates_for_char(
        &mut self,
        registry: &Mutex<FontRegistry>,
        c: char,
        is_emoji_char: bool,
    ) -> Vec<FallbackCandidate> {
        let mut out: Vec<FallbackCandidate> = Vec::with_capacity(3);

        // Emoji: emoji(color) -> symbol(gray) -> system(gray)
        // Non-emoji: system(gray) -> symbol(gray)
        if is_emoji_char {
            self.ensure_emoji_loaded(registry);
            if let Some(face) = self.emoji_font.as_ref() {
                out.push(FallbackCandidate {
                    face: Arc::clone(face),
                    kind: FallbackKind::Emoji,
                    use_color: true,
                });
            }
        }

        self.ensure_symbol_loaded(registry);
        if let Some(face) = self.symbol_font.as_ref() {
            out.push(FallbackCandidate {
                face: Arc::clone(face),
                kind: FallbackKind::Symbol,
                use_color: false,
            });
        }

        if let Some(face) = self.system_fallback_for_char(registry, c) {
            out.push(FallbackCandidate {
                face,
                kind: FallbackKind::System,
                use_color: false,
            });
        }

        if !is_emoji_char {
            // For non-emoji, prefer system over symbol if both exist.
            out.sort_by_key(|c| match c.kind {
                FallbackKind::System => 0,
                FallbackKind::Symbol => 1,
                FallbackKind::Emoji => 2,
            });
        }

        out
    }
}

/// Returns a cache bucket key for the given codepoint.
///
/// This is intentionally a coarse, script-ish bucket (not a 1:1 mapping to
/// Unicode Script). Callers must still verify that a cached fallback face
/// actually contains the specific codepoint and re-resolve if it does not.
pub fn fallback_bucket_key(c: char) -> u32 {
    let cp = c as u32;

    match cp {
        // Hangul (Korean)
        0x1100..=0x11FF // Hangul Jamo
        | 0x3130..=0x318F // Hangul Compatibility Jamo
        | 0xA960..=0xA97F // Hangul Jamo Extended-A
        | 0xAC00..=0xD7A3 // Hangul Syllables
        | 0xD7B0..=0xD7FF // Hangul Jamo Extended-B
        => BUCKET_HANGUL,

        // Hiragana/Katakana (Japanese)
        0x3040..=0x309F | 0x30A0..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9D => BUCKET_KANA,

        // Han ideographs (CJK)
        0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0x20000..=0x2A6DF
        | 0x2A700..=0x2B73F
        | 0x2B740..=0x2B81F
        | 0x2B820..=0x2CEAF
        | 0x2CEB0..=0x2EBEF => BUCKET_HAN,

        // Arabic
        0x0600..=0x06FF
        | 0x0750..=0x077F
        | 0x08A0..=0x08FF
        | 0xFB50..=0xFDFF
        | 0xFE70..=0xFEFF => BUCKET_ARABIC,

        // Devanagari
        0x0900..=0x097F | 0xA8E0..=0xA8FF => BUCKET_DEVANAGARI,

        // Thai
        0x0E00..=0x0E7F => BUCKET_THAI,

        // Hebrew
        0x0590..=0x05FF => BUCKET_HEBREW,

        // Cyrillic
        0x0400..=0x04FF | 0x0500..=0x052F => BUCKET_CYRILLIC,

        // Greek
        0x0370..=0x03FF => BUCKET_GREEK,

        _ => cp,
    }
}
