//! Utilities for "system fallback" font selection.
//!
//! The goal is to avoid repeated full fontdb scans for scripts that have lots of
//! distinct codepoints (Hangul/CJK/etc). We group codepoints into "script-ish"
//! buckets for caching, while still validating that the chosen face covers the
//! exact character at use sites.

use crate::font::FontFace;
use crate::registry::{FontRegistry, GenericFont};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

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

        let resolve = || {
            let mut reg = registry.lock().unwrap();
            reg.load_fallback_for_char(c, self.weight, self.italic)
        };

        let entry = self.sys_cache.entry(bucket).or_insert_with(resolve);
        if let Some(face) = entry.as_ref() {
            if !face.has_glyph(c) {
                *entry = resolve();
            }
        }
        entry.clone()
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
        => 0x11_0000,

        // Hiragana/Katakana (Japanese)
        0x3040..=0x309F | 0x30A0..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9D => 0x11_0001,

        // Han ideographs (CJK)
        0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0x20000..=0x2A6DF
        | 0x2A700..=0x2B73F
        | 0x2B740..=0x2B81F
        | 0x2B820..=0x2CEAF
        | 0x2CEB0..=0x2EBEF => 0x11_0002,

        // Arabic
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF => {
            0x11_0003
        }

        // Devanagari
        0x0900..=0x097F | 0xA8E0..=0xA8FF => 0x11_0004,

        // Thai
        0x0E00..=0x0E7F => 0x11_0005,

        // Hebrew
        0x0590..=0x05FF => 0x11_0006,

        // Cyrillic
        0x0400..=0x04FF | 0x0500..=0x052F => 0x11_0007,

        // Greek
        0x0370..=0x03FF => 0x11_0008,

        _ => cp,
    }
}
