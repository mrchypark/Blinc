//! Font registry for system font discovery and caching
//!
//! Uses fontdb to discover and load system fonts by name or generic category.

use crate::font::FontFace;
use crate::{Result, TextError};
use fontdb::{Database, Family, Query, Source, Stretch, Style, Weight};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Generic font category for fallback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GenericFont {
    /// Default system UI font
    #[default]
    System,
    /// Monospace font for code
    Monospace,
    /// Serif font
    Serif,
    /// Sans-serif font
    SansSerif,
    /// Emoji font (color emoji)
    Emoji,
    /// Symbol font (Unicode symbols, arrows, math, etc.)
    Symbol,
}

/// Font registry that discovers and caches system fonts
pub struct FontRegistry {
    /// fontdb database containing all system fonts
    db: Database,
    /// Cached FontFace instances (Some = found, None = not found)
    faces: FxHashMap<String, Option<Arc<FontFace>>>,
}

impl FontRegistry {
    /// Create a new font registry and load system fonts
    pub fn new() -> Self {
        let mut db = Database::new();

        // Load all system fonts
        db.load_system_fonts();

        let mut registry = Self {
            db,
            faces: FxHashMap::default(),
        };

        // Preload all generic font categories at startup
        registry.preload_generic_fonts();

        registry
    }

    /// Preload all generic font categories
    fn preload_generic_fonts(&mut self) {
        for generic in [
            GenericFont::System,
            GenericFont::SansSerif,
            GenericFont::Serif,
            GenericFont::Monospace,
            GenericFont::Emoji,
            GenericFont::Symbol,
        ] {
            if let Err(e) = self.load_generic(generic) {
                // Emoji/Symbol fonts may not be available on all systems, don't warn
                if generic != GenericFont::Emoji && generic != GenericFont::Symbol {
                    tracing::warn!("Failed to preload generic font {:?}: {:?}", generic, e);
                } else {
                    tracing::debug!("{:?} font not available: {:?}", generic, e);
                }
            }
        }
    }

    /// Preload specific fonts by name with all available variants
    /// (call at startup for fonts your app uses)
    ///
    /// This discovers and loads all variants (bold, italic, etc.) of each font.
    pub fn preload_fonts(&mut self, names: &[&str]) {
        for name in names {
            if self.has_font(name) {
                self.preload_font_family(name);
                tracing::debug!("Preloaded font family with all variants: {}", name);
            } else {
                tracing::debug!("Font not available: {}", name);
            }
        }
    }

    /// Load a font by name (e.g., "Fira Code", "Inter", "Arial")
    pub fn load_font(&mut self, name: &str) -> Result<Arc<FontFace>> {
        self.load_font_with_style(name, 400, false)
    }

    /// Load a font by name with specific weight and italic style
    ///
    /// # Arguments
    /// * `name` - Font family name (e.g., "Fira Code", "Inter")
    /// * `weight` - Font weight (100-900, where 400 is normal, 700 is bold)
    /// * `italic` - Whether to load italic variant
    pub fn load_font_with_style(
        &mut self,
        name: &str,
        weight: u16,
        italic: bool,
    ) -> Result<Arc<FontFace>> {
        // Create cache key that includes weight and style
        let cache_key = format!("{}:w{}:{}", name, weight, if italic { "i" } else { "n" });

        // Check cache first (includes failed lookups as None)
        if let Some(cached) = self.faces.get(&cache_key) {
            return cached.clone().ok_or_else(|| {
                TextError::FontLoadError(format!(
                    "Font '{}' (weight={}, italic={}) not found (cached)",
                    name, weight, italic
                ))
            });
        }

        // Query fontdb for the font by family name with requested weight/style
        let query = Query {
            families: &[Family::Name(name)],
            weight: Weight(weight),
            style: if italic { Style::Italic } else { Style::Normal },
            stretch: Stretch::Normal,
        };

        let id = match self.db.query(&query) {
            Some(id) => id,
            None => {
                // Try with Oblique if Italic wasn't found
                if italic {
                    let oblique_query = Query {
                        families: &[Family::Name(name)],
                        weight: Weight(weight),
                        style: Style::Oblique,
                        stretch: Stretch::Normal,
                    };
                    match self.db.query(&oblique_query) {
                        Some(id) => id,
                        None => {
                            self.faces.insert(cache_key.clone(), None);
                            return Err(TextError::FontLoadError(format!(
                                "Font '{}' (weight={}, italic={}) not found",
                                name, weight, italic
                            )));
                        }
                    }
                } else {
                    self.faces.insert(cache_key.clone(), None);
                    return Err(TextError::FontLoadError(format!(
                        "Font '{}' (weight={}, italic={}) not found",
                        name, weight, italic
                    )));
                }
            }
        };

        // Get the font data
        let face = self.load_face_by_id(id)?;
        let face = Arc::new(face);

        // Cache it
        self.faces.insert(cache_key, Some(Arc::clone(&face)));

        Ok(face)
    }

    /// Load the system emoji font
    ///
    /// Tries platform-specific emoji fonts:
    /// - macOS: Apple Color Emoji
    /// - Windows: Segoe UI Emoji
    /// - Linux: Noto Color Emoji, Noto Emoji, Twitter Color Emoji
    fn load_emoji_font(&mut self) -> Result<Arc<FontFace>> {
        let cache_key = "__generic_Emoji:w400:n".to_string();

        // Check cache first
        if let Some(cached) = self.faces.get(&cache_key) {
            return cached.clone().ok_or_else(|| {
                TextError::FontLoadError("Emoji font not found (cached)".to_string())
            });
        }

        // Platform-specific emoji and symbol font names to try
        // These fonts provide coverage for emoji and special Unicode symbols
        let emoji_fonts = if cfg!(target_os = "macos") {
            vec![
                "Apple Color Emoji", // Color emoji
                "Apple Symbols",     // Unicode symbols (arrows, math, etc.)
            ]
        } else if cfg!(target_os = "windows") {
            vec![
                "Segoe UI Emoji",  // Color emoji
                "Segoe UI Symbol", // Unicode symbols
                "Segoe UI",        // Additional symbol coverage
            ]
        } else {
            // Linux and others
            vec![
                "Noto Color Emoji",   // Color emoji
                "Noto Emoji",         // Monochrome emoji fallback
                "Noto Sans Symbols",  // Unicode symbols
                "Noto Sans Symbols2", // Additional symbols
                "Twitter Color Emoji",
                "EmojiOne Color",
                "JoyPixels",
                "DejaVu Sans", // Good Unicode coverage
            ]
        };

        for font_name in emoji_fonts {
            if let Ok(face) = self.load_font(font_name) {
                // Cache it under the generic emoji key
                self.faces.insert(cache_key.clone(), Some(Arc::clone(&face)));
                tracing::debug!("Loaded emoji font: {}", font_name);
                return Ok(face);
            }
        }

        // No emoji font found
        self.faces.insert(cache_key, None);
        Err(TextError::FontLoadError(
            "No emoji font found on system".to_string(),
        ))
    }

    /// Load the system symbol font
    ///
    /// Tries platform-specific symbol fonts for Unicode symbols:
    /// - macOS: Menlo (has dingbats like ✓✗), Apple Symbols
    /// - Windows: Segoe UI Symbol
    /// - Linux: Noto Sans Symbols, DejaVu Sans
    fn load_symbol_font(&mut self) -> Result<Arc<FontFace>> {
        let cache_key = "__generic_Symbol:w400:n".to_string();

        // Check cache first
        if let Some(cached) = self.faces.get(&cache_key) {
            return cached.clone().ok_or_else(|| {
                TextError::FontLoadError("Symbol font not found (cached)".to_string())
            });
        }

        // Platform-specific symbol font names to try
        // Priority: fonts with good dingbat/symbol coverage (✓, ✗, etc.) first
        let symbol_fonts = if cfg!(target_os = "macos") {
            vec![
                "Menlo",            // Has ✓ ✗ ✔ ✖ and other dingbats
                "Lucida Grande",    // Has ✓ and many symbols
                "Apple Symbols",    // Unicode symbols (arrows, math, but NOT ✓✗)
            ]
        } else if cfg!(target_os = "windows") {
            vec![
                "Segoe UI Symbol", // Unicode symbols
                "Segoe UI",        // Additional symbol coverage
                "Arial Unicode MS",
            ]
        } else {
            // Linux and others
            vec![
                "DejaVu Sans",        // Good Unicode coverage including dingbats
                "Noto Sans Symbols",  // Unicode symbols
                "Noto Sans Symbols2", // Additional symbols
                "FreeSans",
            ]
        };

        for font_name in symbol_fonts {
            if let Ok(face) = self.load_font(font_name) {
                // Cache it under the generic symbol key
                self.faces.insert(cache_key.clone(), Some(Arc::clone(&face)));
                tracing::debug!("Loaded symbol font: {}", font_name);
                return Ok(face);
            }
        }

        // No symbol font found
        self.faces.insert(cache_key, None);
        Err(TextError::FontLoadError(
            "No symbol font found on system".to_string(),
        ))
    }

    /// Load a font face by fontdb ID
    fn load_face_by_id(&self, id: fontdb::ID) -> Result<FontFace> {
        // Get the face source info
        let (src, face_index) = self
            .db
            .face_source(id)
            .ok_or_else(|| TextError::FontLoadError("Font source not found".to_string()))?;

        // Load the font data
        let data = match src {
            Source::File(path) => std::fs::read(&path).map_err(|e| {
                TextError::FontLoadError(format!("Failed to read font file {:?}: {}", path, e))
            })?,
            Source::Binary(arc) => arc.as_ref().as_ref().to_vec(),
            Source::SharedFile(_path, data) => data.as_ref().as_ref().to_vec(),
        };

        // Create FontFace with the correct index
        FontFace::from_data_with_index(data, face_index)
    }

    /// Load a generic font category
    pub fn load_generic(&mut self, generic: GenericFont) -> Result<Arc<FontFace>> {
        self.load_generic_with_style(generic, 400, false)
    }

    /// Load a generic font category with specific weight and italic style
    ///
    /// # Arguments
    /// * `generic` - Generic font category (System, Monospace, Serif, SansSerif)
    /// * `weight` - Font weight (100-900, where 400 is normal, 700 is bold)
    /// * `italic` - Whether to load italic variant
    pub fn load_generic_with_style(
        &mut self,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> Result<Arc<FontFace>> {
        let cache_key = format!(
            "__generic_{:?}:w{}:{}",
            generic,
            weight,
            if italic { "i" } else { "n" }
        );

        // Check cache first (includes failed lookups as None)
        if let Some(cached) = self.faces.get(&cache_key) {
            return cached.clone().ok_or_else(|| {
                TextError::FontLoadError(format!(
                    "Generic font {:?} (weight={}, italic={}) not found (cached)",
                    generic, weight, italic
                ))
            });
        }

        // Map GenericFont to fontdb Family
        // For Emoji and Symbol, we try platform-specific fonts by name
        if generic == GenericFont::Emoji {
            return self.load_emoji_font();
        }
        if generic == GenericFont::Symbol {
            return self.load_symbol_font();
        }

        let family = match generic {
            GenericFont::System => Family::SansSerif,
            GenericFont::Monospace => Family::Monospace,
            GenericFont::Serif => Family::Serif,
            GenericFont::SansSerif => Family::SansSerif,
            GenericFont::Emoji => unreachable!(),  // Handled above
            GenericFont::Symbol => unreachable!(), // Handled above
        };

        // Query fontdb with requested weight and style
        let query = Query {
            families: &[family],
            weight: Weight(weight),
            style: if italic { Style::Italic } else { Style::Normal },
            stretch: Stretch::Normal,
        };

        let id = match self.db.query(&query) {
            Some(id) => id,
            None => {
                // Try with Oblique if Italic wasn't found
                if italic {
                    let oblique_query = Query {
                        families: &[family],
                        weight: Weight(weight),
                        style: Style::Oblique,
                        stretch: Stretch::Normal,
                    };
                    match self.db.query(&oblique_query) {
                        Some(id) => id,
                        None => {
                            self.faces.insert(cache_key.clone(), None);
                            return Err(TextError::FontLoadError(format!(
                                "Generic font {:?} (weight={}, italic={}) not found",
                                generic, weight, italic
                            )));
                        }
                    }
                } else {
                    self.faces.insert(cache_key.clone(), None);
                    return Err(TextError::FontLoadError(format!(
                        "Generic font {:?} (weight={}, italic={}) not found",
                        generic, weight, italic
                    )));
                }
            }
        };

        let face = self.load_face_by_id(id)?;
        let face = Arc::new(face);

        // Cache it
        self.faces.insert(cache_key, Some(Arc::clone(&face)));

        Ok(face)
    }

    /// Load a font with fallback to generic category
    pub fn load_with_fallback(
        &mut self,
        name: Option<&str>,
        generic: GenericFont,
    ) -> Result<Arc<FontFace>> {
        self.load_with_fallback_styled(name, generic, 400, false)
    }

    /// Load a font with fallback to generic category, with specific weight and style
    pub fn load_with_fallback_styled(
        &mut self,
        name: Option<&str>,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> Result<Arc<FontFace>> {
        // Try named font first
        if let Some(name) = name {
            // Check if we've already tried this font (avoid repeated warnings)
            let cache_key = format!("{}:w{}:{}", name, weight, if italic { "i" } else { "n" });
            let already_tried = self.faces.contains_key(&cache_key);

            tracing::trace!(
                "load_with_fallback_styled: name={}, weight={}, italic={}, already_tried={}, cache_size={}",
                name,
                weight,
                italic,
                already_tried,
                self.faces.len()
            );

            if let Ok(face) = self.load_font_with_style(name, weight, italic) {
                return Ok(face);
            }

            // Only warn on the first failure for this font
            if !already_tried {
                tracing::warn!(
                    "Font '{}' (weight={}, italic={}) not found, falling back to {:?}",
                    name,
                    weight,
                    italic,
                    generic
                );
            }
        }

        // Fall back to generic with same style
        self.load_generic_with_style(generic, weight, italic)
    }

    /// Get cached font by name (doesn't load - for use during render)
    pub fn get_cached(&self, name: &str) -> Option<Arc<FontFace>> {
        // Legacy: check for normal weight/style first
        let cache_key = format!("{}:w400:n", name);
        self.faces.get(&cache_key).and_then(|opt| opt.clone())
    }

    /// Get cached font by name with specific weight and style
    pub fn get_cached_with_style(
        &self,
        name: &str,
        weight: u16,
        italic: bool,
    ) -> Option<Arc<FontFace>> {
        let cache_key = format!("{}:w{}:{}", name, weight, if italic { "i" } else { "n" });
        self.faces.get(&cache_key).and_then(|opt| opt.clone())
    }

    /// Get cached generic font (doesn't load - for use during render)
    pub fn get_cached_generic(&self, generic: GenericFont) -> Option<Arc<FontFace>> {
        // Legacy: check for normal weight/style first
        let cache_key = format!("__generic_{:?}:w400:n", generic);
        self.faces.get(&cache_key).and_then(|opt| opt.clone())
    }

    /// Get cached generic font with specific weight and style
    pub fn get_cached_generic_with_style(
        &self,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> Option<Arc<FontFace>> {
        let cache_key = format!(
            "__generic_{:?}:w{}:{}",
            generic,
            weight,
            if italic { "i" } else { "n" }
        );
        self.faces.get(&cache_key).and_then(|opt| opt.clone())
    }

    /// Fast font lookup for rendering - only uses cache, never loads
    /// Returns the requested font if cached, or None if loading is needed
    pub fn get_for_render(
        &self,
        name: Option<&str>,
        generic: GenericFont,
    ) -> Option<Arc<FontFace>> {
        self.get_for_render_with_style(name, generic, 400, false)
    }

    /// Fast font lookup for rendering with specific weight and style
    pub fn get_for_render_with_style(
        &self,
        name: Option<&str>,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> Option<Arc<FontFace>> {
        // Try named font from cache first
        if let Some(name) = name {
            // For named fonts, only return if we have that specific font cached
            // Return None to trigger loading if not found
            return self.get_cached_with_style(name, weight, italic);
        }

        // For generic-only requests, use cached generic font with style
        self.get_cached_generic_with_style(generic, weight, italic)
            .or_else(|| self.get_cached_generic_with_style(GenericFont::SansSerif, weight, italic))
    }

    /// Get the emoji font if available (cached)
    ///
    /// Returns the cached emoji font if it was successfully loaded during
    /// initialization, or None if no emoji font is available.
    pub fn get_emoji_font(&self) -> Option<Arc<FontFace>> {
        self.get_cached_generic(GenericFont::Emoji)
    }

    /// Check if a character needs an emoji font
    ///
    /// Returns true for emoji characters that typically need a color emoji font.
    pub fn needs_emoji_font(c: char) -> bool {
        crate::emoji::is_emoji(c)
    }

    /// List available font families on the system
    pub fn list_families(&self) -> Vec<String> {
        let mut families: Vec<String> = self
            .db
            .faces()
            .filter_map(|face| face.families.first().map(|(name, _)| name.clone()))
            .collect();

        families.sort();
        families.dedup();
        families
    }

    /// Check if a font is available
    pub fn has_font(&self, name: &str) -> bool {
        let query = Query {
            families: &[Family::Name(name)],
            weight: Weight::NORMAL,
            style: Style::Normal,
            stretch: Stretch::Normal,
        };
        self.db.query(&query).is_some()
    }

    /// Preload all variants (weights and styles) of a font family
    ///
    /// This discovers all available variants of the font using fontdb
    /// and loads each one into the cache.
    pub fn preload_font_family(&mut self, name: &str) {
        // Find all faces that belong to this font family
        let face_ids: Vec<_> = self
            .db
            .faces()
            .filter(|face| {
                face.families
                    .iter()
                    .any(|(family_name, _)| family_name == name)
            })
            .map(|face| (face.id, face.weight.0, face.style))
            .collect();

        // Load each variant
        for (id, weight, style) in face_ids {
            let italic = matches!(style, Style::Italic | Style::Oblique);
            let cache_key = format!("{}:w{}:{}", name, weight, if italic { "i" } else { "n" });

            // Skip if already cached
            if self.faces.contains_key(&cache_key) {
                continue;
            }

            // Load the face
            match self.load_face_by_id(id) {
                Ok(face) => {
                    self.faces.insert(cache_key, Some(Arc::new(face)));
                }
                Err(e) => {
                    tracing::warn!("Failed to load font variant {}: {:?}", cache_key, e);
                    self.faces.insert(cache_key, None);
                }
            }
        }
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_generic_fonts() {
        let mut registry = FontRegistry::new();

        // Try to load generic fonts - may not be available in minimal CI environments
        let sans = registry.load_generic(GenericFont::SansSerif);
        let mono = registry.load_generic(GenericFont::Monospace);

        // At least one generic font should be available on most systems
        if sans.is_err() && mono.is_err() {
            println!("No generic fonts available - skipping test (CI environment)");
            return;
        }

        // If we have fonts, verify they loaded correctly
        if let Ok(font) = sans {
            println!("Loaded sans-serif: {}", font.family_name());
        }
        if let Ok(font) = mono {
            println!("Loaded monospace: {}", font.family_name());
        }
    }

    #[test]
    fn test_list_families() {
        let registry = FontRegistry::new();
        let families = registry.list_families();
        // May be empty in minimal CI environments without fonts
        println!("Found {} font families", families.len());
        if families.is_empty() {
            println!("No fonts found - likely minimal CI environment");
        }
    }

    #[test]
    fn test_check_cross_glyph_coverage() {
        let mut registry = FontRegistry::new();

        // Test characters: check and cross marks
        let test_chars = ['✓', '✗', '✔', '✖'];

        println!("\n=== Testing glyph coverage for check/cross marks ===\n");

        // Test common fonts
        let fonts_to_test = [
            "Arial", "Helvetica", "Helvetica Neue", "SF Pro", "SF Pro Text",
            "Menlo", "Monaco", "Lucida Grande", "Times New Roman",
            "Apple Symbols", "Apple Color Emoji",
        ];

        for font_name in fonts_to_test {
            match registry.load_font(font_name) {
                Ok(font) => {
                    print!("{:20} ", font_name);
                    for c in test_chars {
                        let glyph_id = font.glyph_id(c);
                        match glyph_id {
                            Some(id) if id > 0 => print!("'{}':✓({}) ", c, id),
                            _ => print!("'{}':✗     ", c),
                        }
                    }
                    println!();
                }
                Err(_) => {
                    println!("{:20} NOT AVAILABLE", font_name);
                }
            }
        }

        // Test generic fonts
        println!("\n--- Generic Fonts ---");

        if let Ok(font) = registry.load_generic(GenericFont::System) {
            print!("{:20} ", format!("System ({})", font.family_name()));
            for c in test_chars {
                let glyph_id = font.glyph_id(c);
                match glyph_id {
                    Some(id) if id > 0 => print!("'{}':✓({}) ", c, id),
                    _ => print!("'{}':✗     ", c),
                }
            }
            println!();
        }

        if let Ok(font) = registry.load_generic(GenericFont::Symbol) {
            print!("{:20} ", format!("Symbol ({})", font.family_name()));
            for c in test_chars {
                let glyph_id = font.glyph_id(c);
                match glyph_id {
                    Some(id) if id > 0 => print!("'{}':✓({}) ", c, id),
                    _ => print!("'{}':✗     ", c),
                }
            }
            println!();
        }
    }

    #[test]
    fn test_list_all_fonts_with_check() {
        let mut registry = FontRegistry::new();
        let families = registry.list_families();

        println!("\n=== Fonts with ✓ (U+2713) glyph ===\n");

        let mut fonts_with_check = Vec::new();
        for family in &families {
            if let Ok(font) = registry.load_font(family) {
                if let Some(gid) = font.glyph_id('✓') {
                    if gid > 0 {
                        fonts_with_check.push((family.clone(), gid));
                    }
                }
            }
        }

        for (name, gid) in &fonts_with_check {
            println!("{}: glyph_id={}", name, gid);
        }
        println!("\nTotal fonts with ✓: {}", fonts_with_check.len());
    }

    #[test]
    fn test_menlo_font_loading() {
        let mut registry = FontRegistry::new();

        // Try to load Menlo
        match registry.load_font("Menlo") {
            Ok(font) => {
                println!("\n=== Menlo Font Info ===");
                println!("Family name: {}", font.family_name());
                println!("Face index: {}", font.face_index());
                println!("Weight: {:?}", font.weight());
                println!("Style: {:?}", font.style());
                println!("Glyph count: {}", font.glyph_count());

                // Test some glyph IDs
                for c in ['A', 'F', 'S', 'M', 'i', 'n', 'l'] {
                    if let Some(id) = font.glyph_id(c) {
                        println!("  '{}' -> glyph_id {}", c, id);
                    } else {
                        println!("  '{}' -> NOT FOUND", c);
                    }
                }
            }
            Err(e) => {
                println!("Failed to load Menlo: {:?}", e);
            }
        }
    }

    #[test]
    fn test_sf_mono_loading() {
        let mut registry = FontRegistry::new();

        // Try to load SF Mono
        match registry.load_font("SF Mono") {
            Ok(font) => {
                println!("\n=== SF Mono Font Info ===");
                println!("Family name: {}", font.family_name());
                println!("Face index: {}", font.face_index());
                println!("Weight: {:?}", font.weight());
                println!("Style: {:?}", font.style());
                println!("Glyph count: {}", font.glyph_count());

                // Test glyph IDs for "SF" - these should NOT be the same as "SI"
                println!("\nGlyph mappings:");
                for c in ['S', 'F', 'I', ' ', 'M', 'o', 'n'] {
                    if let Some(id) = font.glyph_id(c) {
                        println!("  '{}' (U+{:04X}) -> glyph_id {}", c, c as u32, id);
                    } else {
                        println!("  '{}' -> NOT FOUND", c);
                    }
                }
            }
            Err(e) => {
                println!("SF Mono not available: {:?}", e);
            }
        }
    }

    #[test]
    fn test_text_shaping() {
        use crate::shaper::TextShaper;

        let mut registry = FontRegistry::new();
        let shaper = TextShaper::new();

        // Try to load a font - SF Mono, then monospace, then any available
        let font = match registry.load_font("SF Mono") {
            Ok(f) => f,
            Err(_) => match registry.load_generic(GenericFont::Monospace) {
                Ok(f) => f,
                Err(_) => match registry.load_generic(GenericFont::SansSerif) {
                    Ok(f) => f,
                    Err(_) => {
                        println!("No fonts available - skipping test (CI environment)");
                        return;
                    }
                },
            },
        };

        println!("\n=== Testing text shaping ===");
        println!(
            "Using font: {} (face_index={})",
            font.family_name(),
            font.face_index()
        );

        // Shape the text "SF"
        let shaped = shaper.shape("SF", &font, 24.0);

        println!("Shaped 'SF' -> {} glyphs:", shaped.glyphs.len());
        for (i, glyph) in shaped.glyphs.iter().enumerate() {
            println!(
                "  [{}] glyph_id={}, x_advance={}, cluster={}",
                i, glyph.glyph_id, glyph.x_advance, glyph.cluster
            );
        }

        // The glyph IDs for 'S' and 'F' should be different
        if shaped.glyphs.len() >= 2 {
            let s_glyph = shaped.glyphs[0].glyph_id;
            let f_glyph = shaped.glyphs[1].glyph_id;
            println!("\nS glyph_id: {}, F glyph_id: {}", s_glyph, f_glyph);
            assert_ne!(s_glyph, f_glyph, "S and F should have different glyph IDs");
        }
    }

    #[test]
    fn test_full_text_rendering() {
        use crate::layout::LayoutOptions;
        use crate::renderer::TextRenderer;

        let mut renderer = TextRenderer::new();

        // Preload SF Mono
        renderer.preload_fonts(&["SF Mono"]);

        println!("\n=== Testing full text rendering for 'SF Mono' ===");

        // Prepare text through the full pipeline
        let options = LayoutOptions::default();
        let result = renderer.prepare_text_with_font(
            "SF Mono",
            24.0,
            [0.0, 0.0, 0.0, 1.0],
            &options,
            Some("SF Mono"),
            GenericFont::Monospace,
        );

        match result {
            Ok(prepared) => {
                println!("Prepared {} glyphs for 'SF Mono':", prepared.glyphs.len());
                for (i, glyph) in prepared.glyphs.iter().enumerate() {
                    println!("  [{}] bounds=[{:.1}, {:.1}, {:.1}, {:.1}], uv=[{:.4}, {:.4}, {:.4}, {:.4}]",
                        i, glyph.bounds[0], glyph.bounds[1], glyph.bounds[2], glyph.bounds[3],
                        glyph.uv_bounds[0], glyph.uv_bounds[1], glyph.uv_bounds[2], glyph.uv_bounds[3]);
                }
            }
            Err(e) => {
                println!("Error preparing text: {:?}", e);
            }
        }
    }

    #[test]
    fn test_fira_code_loading() {
        let mut registry = FontRegistry::new();

        // Try to load Fira Code
        match registry.load_font("Fira Code") {
            Ok(font) => {
                println!("\n=== Fira Code Font Info ===");
                println!("Family name: {}", font.family_name());
                println!("Face index: {}", font.face_index());
                println!("Weight: {:?}", font.weight());
                println!("Style: {:?}", font.style());
                println!("Glyph count: {}", font.glyph_count());

                // Test glyph IDs - specifically F and B which should be different
                println!("\nGlyph mappings:");
                for c in ['F', 'B', 'i', 'r', 'a', ' ', 'C', 'o', 'd', 'e'] {
                    if let Some(id) = font.glyph_id(c) {
                        println!("  '{}' (U+{:04X}) -> glyph_id {}", c, c as u32, id);
                    } else {
                        println!("  '{}' -> NOT FOUND", c);
                    }
                }
            }
            Err(e) => {
                println!("Fira Code not available: {:?}", e);
            }
        }
    }
}
