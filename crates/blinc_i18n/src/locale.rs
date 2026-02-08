/// Normalize locale identifiers to a canonical-ish form for lookup.
///
/// - Converts `_` to `-` (Android often reports `en_US`).
/// - Trims whitespace.
pub fn normalize_locale(s: &str) -> String {
    s.trim().replace('_', "-")
}

/// Create a fallback chain for translation lookup.
///
/// Example:
/// - `ko-KR` -> `["ko-KR", "ko", "en-US"]`
/// - `en-US` -> `["en-US", "en", "en-US"]` (deduped to `["en-US", "en"]`)
pub fn locale_fallback_chain(locale: &str) -> Vec<String> {
    let l = normalize_locale(locale);
    let mut chain = Vec::new();

    if !l.is_empty() {
        chain.push(l.clone());
        if let Some(lang) = l.split('-').next() {
            if !lang.is_empty() {
                chain.push(lang.to_string());
            }
        }
    }

    // Hard fallback for now: English.
    chain.push("en-US".to_string());

    // Dedup, preserve order.
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for x in chain {
        if seen.insert(x.clone()) {
            out.push(x);
        }
    }
    out
}
