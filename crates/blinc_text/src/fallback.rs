//! Utilities for "system fallback" font selection.
//!
//! The goal is to avoid repeated full fontdb scans for scripts that have lots of
//! distinct codepoints (Hangul/CJK/etc). We group codepoints into "script-ish"
//! buckets for caching, while still validating that the chosen face covers the
//! exact character at use sites.

/// Returns a cache bucket key for the given codepoint.
///
/// This is intentionally a coarse, script-ish bucket (not a 1:1 mapping to
/// Unicode Script). Callers must still verify that a cached fallback face
/// actually contains the specific codepoint and re-resolve if it does not.
pub fn fallback_bucket_key(c: char) -> u32 {
    let cp = c as u32;

    // Hangul (Korean)
    if (0x1100..=0x11FF).contains(&cp) // Hangul Jamo
        || (0x3130..=0x318F).contains(&cp) // Hangul Compatibility Jamo
        || (0xA960..=0xA97F).contains(&cp) // Hangul Jamo Extended-A
        || (0xAC00..=0xD7A3).contains(&cp) // Hangul Syllables
        || (0xD7B0..=0xD7FF).contains(&cp)
    // Hangul Jamo Extended-B
    {
        return 0x11_0000;
    }

    // Hiragana/Katakana (Japanese)
    if (0x3040..=0x309F).contains(&cp)
        || (0x30A0..=0x30FF).contains(&cp)
        || (0x31F0..=0x31FF).contains(&cp)
        || (0xFF66..=0xFF9D).contains(&cp)
    {
        return 0x11_0001;
    }

    // Han ideographs (CJK)
    if (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0x20000..=0x2A6DF).contains(&cp)
        || (0x2A700..=0x2B73F).contains(&cp)
        || (0x2B740..=0x2B81F).contains(&cp)
        || (0x2B820..=0x2CEAF).contains(&cp)
        || (0x2CEB0..=0x2EBEF).contains(&cp)
    {
        return 0x11_0002;
    }

    // Arabic
    if (0x0600..=0x06FF).contains(&cp)
        || (0x0750..=0x077F).contains(&cp)
        || (0x08A0..=0x08FF).contains(&cp)
        || (0xFB50..=0xFDFF).contains(&cp)
        || (0xFE70..=0xFEFF).contains(&cp)
    {
        return 0x11_0003;
    }

    // Devanagari
    if (0x0900..=0x097F).contains(&cp) || (0xA8E0..=0xA8FF).contains(&cp) {
        return 0x11_0004;
    }

    // Thai
    if (0x0E00..=0x0E7F).contains(&cp) {
        return 0x11_0005;
    }

    // Hebrew
    if (0x0590..=0x05FF).contains(&cp) {
        return 0x11_0006;
    }

    // Cyrillic
    if (0x0400..=0x04FF).contains(&cp) || (0x0500..=0x052F).contains(&cp) {
        return 0x11_0007;
    }

    // Greek
    if (0x0370..=0x03FF).contains(&cp) {
        return 0x11_0008;
    }

    cp
}
