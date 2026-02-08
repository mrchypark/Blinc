//! Text renderer
//!
//! High-level text rendering that combines font loading, shaping,
//! rasterization, atlas management, and glyph instance generation.
//!
//! Supports automatic emoji font fallback - when the primary font doesn't
//! have a glyph for an emoji character, the system emoji font is used.

use crate::atlas::{ColorGlyphAtlas, GlyphAtlas, GlyphInfo};
use crate::emoji::is_emoji;
use crate::font::{FontFace, FontStyle};
use crate::layout::{LayoutOptions, PositionedGlyph, TextLayout, TextLayoutEngine};
use crate::rasterizer::GlyphRasterizer;
use crate::registry::{FontRegistry, GenericFont};
use crate::{Result, TextError};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Maximum number of glyphs to keep in the grayscale glyph cache
const GLYPH_CACHE_CAPACITY: usize = 2048;

/// Maximum number of color glyphs (emoji) to keep in cache
const COLOR_GLYPH_CACHE_CAPACITY: usize = 512;

/// A GPU glyph instance for rendering
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    /// Position and size in pixels (x, y, width, height)
    pub bounds: [f32; 4],
    /// UV coordinates in atlas (u_min, v_min, u_max, v_max)
    pub uv_bounds: [f32; 4],
    /// Text color (RGBA, 0.0-1.0)
    pub color: [f32; 4],
    /// Whether this glyph is from the color atlas (emoji)
    pub is_color: bool,
}

/// Result of preparing text for rendering
#[derive(Debug)]
pub struct PreparedText {
    /// Glyph instances ready for GPU rendering
    pub glyphs: Vec<GlyphInstance>,
    /// Total width of the text
    pub width: f32,
    /// Total height of the text (line height)
    pub height: f32,
    /// Ascender in pixels (distance from baseline to top of em box)
    pub ascender: f32,
    /// Descender in pixels (typically negative, distance from baseline to bottom)
    pub descender: f32,
}

/// A color span for styled text rendering
#[derive(Debug, Clone, Copy)]
pub struct ColorSpan {
    /// Start byte index in text
    pub start: usize,
    /// End byte index in text (exclusive)
    pub end: usize,
    /// RGBA color
    pub color: [f32; 4],
}

struct ResolvedGlyphData {
    info: GlyphInfo,
    positioned: PositionedGlyph,
    is_color: bool,
}

struct RenderFallbackWalker<'a> {
    renderer: &'a mut TextRenderer,
    glyph_infos: &'a mut Vec<Option<ResolvedGlyphData>>,
    font: &'a FontFace,
    font_id: u32,
    font_size: f32,
    options: &'a LayoutOptions,

    fallback_font_id_cache: &'a mut rustc_hash::FxHashMap<usize, u32>,
    gid_resolver: &'a mut crate::fallback::FallbackGlyphIdResolver,
}

impl crate::fallback::FallbackWalkHandler for RenderFallbackWalker<'_> {
    type Error = TextError;

    fn on_skip(&mut self) -> std::result::Result<(), Self::Error> {
        self.glyph_infos.push(None);
        Ok(())
    }

    fn on_primary(&mut self, glyph: PositionedGlyph) -> std::result::Result<(), Self::Error> {
        let glyph_info = self.renderer.rasterize_glyph_for_font(
            self.font,
            self.font_id,
            glyph.glyph_id,
            self.font_size,
        )?;
        self.glyph_infos.push(Some(ResolvedGlyphData {
            info: glyph_info,
            positioned: glyph,
            is_color: false,
        }));
        Ok(())
    }

    fn on_fallback(
        &mut self,
        glyph: PositionedGlyph,
        candidate: &crate::fallback::FallbackCandidate,
    ) -> std::result::Result<Option<f32>, Self::Error> {
        let Some(nominal_gid) = candidate.face.glyph_id(glyph.codepoint) else {
            return Ok(None);
        };
        if nominal_gid == 0 {
            return Ok(None);
        }

        let is_emoji_char = is_emoji(glyph.codepoint);

        let fallback_gid =
            self.gid_resolver
                .resolve_gid(candidate, glyph.codepoint, self.font_size, nominal_gid);

        let fallback_font_id = match candidate.kind {
            crate::fallback::FallbackKind::Emoji => self.renderer.font_id(None, GenericFont::Emoji),
            crate::fallback::FallbackKind::Symbol => {
                self.renderer.font_id(None, GenericFont::Symbol)
            }
            crate::fallback::FallbackKind::System => {
                let key = Arc::as_ptr(&candidate.face) as usize;
                *self
                    .fallback_font_id_cache
                    .entry(key)
                    .or_insert_with(|| self.renderer.font_id_for_fallback_face(&candidate.face))
            }
        };

        let (glyph_info, is_color) = if candidate.use_color && is_emoji_char {
            let info = self.renderer.rasterize_color_glyph_for_font(
                &candidate.face,
                fallback_font_id,
                fallback_gid,
                self.font_size,
            )?;
            (info, true)
        } else {
            let info = self.renderer.rasterize_glyph_for_font(
                &candidate.face,
                fallback_font_id,
                fallback_gid,
                self.font_size,
            )?;
            (info, false)
        };

        let fallback_positioned = PositionedGlyph {
            glyph_id: fallback_gid,
            cluster: glyph.cluster,
            codepoint: glyph.codepoint,
            x: glyph.x,
            y: glyph.y,
        };

        let fallback_advance = glyph_info.advance as f32 + self.options.letter_spacing;
        self.glyph_infos.push(Some(ResolvedGlyphData {
            info: glyph_info,
            positioned: fallback_positioned,
            is_color,
        }));

        Ok(Some(fallback_advance))
    }
}

/// Text renderer that manages fonts, atlas, and glyph rendering
pub struct TextRenderer {
    /// Default font (legacy support)
    default_font: Option<FontFace>,
    /// Font registry for system font discovery and caching
    /// Can be shared with other components (like text measurement)
    font_registry: Arc<std::sync::Mutex<FontRegistry>>,
    /// Glyph atlas (grayscale for regular text)
    atlas: GlyphAtlas,
    /// Color glyph atlas (RGBA for color emoji)
    color_atlas: ColorGlyphAtlas,
    /// Glyph rasterizer
    rasterizer: GlyphRasterizer,
    /// Text layout engine
    layout_engine: TextLayoutEngine,
    /// LRU cache for grayscale glyphs: (font_id, glyph_id, quantized_size) -> atlas info
    /// font_id is hash of font name or 0 for default
    glyph_cache: LruCache<(u32, u16, u16), GlyphInfo>,
    /// LRU cache for color glyphs (emoji) - same key format
    color_glyph_cache: LruCache<(u32, u16, u16), GlyphInfo>,
}

impl TextRenderer {
    fn font_id_for_fallback_face(&self, font: &FontFace) -> u32 {
        use rustc_hash::FxHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = FxHasher::default();
        "__fallback".hash(&mut hasher);
        font.family_name().hash(&mut hasher);
        font.face_index().hash(&mut hasher);
        font.weight().to_number().hash(&mut hasher);
        font.style().hash(&mut hasher);
        hasher.finish() as u32
    }

    fn build_glyph_infos_with_fallback(
        &mut self,
        layout: &TextLayout,
        font: &FontFace,
        font_id: u32,
        font_size: f32,
        options: &LayoutOptions,
        weight: u16,
        italic: bool,
    ) -> Result<(Vec<Option<ResolvedGlyphData>>, f32)> {
        let registry = Arc::clone(&self.font_registry);
        let mut gid_resolver = crate::fallback::FallbackGlyphIdResolver::new();

        // Cache computed system fallback face IDs to reduce hashing overhead.
        let mut fallback_font_id_cache: rustc_hash::FxHashMap<usize, u32> =
            rustc_hash::FxHashMap::default();

        let mut glyph_infos: Vec<Option<ResolvedGlyphData>> =
            Vec::with_capacity(layout.glyph_count());

        let mut walker = RenderFallbackWalker {
            renderer: self,
            glyph_infos: &mut glyph_infos,
            font,
            font_id,
            font_size,
            options,
            fallback_font_id_cache: &mut fallback_font_id_cache,
            gid_resolver: &mut gid_resolver,
        };

        let corrected_width = crate::fallback::walk_layout_with_fallback(
            layout,
            font,
            registry.as_ref(),
            weight,
            italic,
            &mut walker,
        )?;

        Ok((glyph_infos, corrected_width))
    }

    /// Create a new text renderer with default atlas size.
    ///
    /// Uses the global shared font registry to minimize memory usage.
    /// Apple Color Emoji alone is 180MB - sharing prevents loading it multiple times.
    pub fn new() -> Self {
        Self {
            default_font: None,
            font_registry: crate::global_font_registry(),
            atlas: GlyphAtlas::default(),
            color_atlas: ColorGlyphAtlas::default(),
            rasterizer: GlyphRasterizer::new(),
            layout_engine: TextLayoutEngine::new(),
            glyph_cache: LruCache::new(NonZeroUsize::new(GLYPH_CACHE_CAPACITY).unwrap()),
            color_glyph_cache: LruCache::new(
                NonZeroUsize::new(COLOR_GLYPH_CACHE_CAPACITY).unwrap(),
            ),
        }
    }

    /// Create a new text renderer with a shared font registry
    ///
    /// Use this to share fonts between text measurement and rendering,
    /// ensuring consistent text layout.
    pub fn with_shared_registry(registry: Arc<std::sync::Mutex<FontRegistry>>) -> Self {
        Self {
            default_font: None,
            font_registry: registry,
            atlas: GlyphAtlas::default(),
            color_atlas: ColorGlyphAtlas::default(),
            rasterizer: GlyphRasterizer::new(),
            layout_engine: TextLayoutEngine::new(),
            glyph_cache: LruCache::new(NonZeroUsize::new(GLYPH_CACHE_CAPACITY).unwrap()),
            color_glyph_cache: LruCache::new(
                NonZeroUsize::new(COLOR_GLYPH_CACHE_CAPACITY).unwrap(),
            ),
        }
    }

    /// Create with custom atlas size.
    ///
    /// Uses the global shared font registry to minimize memory usage.
    pub fn with_atlas_size(width: u32, height: u32) -> Self {
        Self {
            default_font: None,
            font_registry: crate::global_font_registry(),
            atlas: GlyphAtlas::new(width, height),
            color_atlas: ColorGlyphAtlas::default(),
            rasterizer: GlyphRasterizer::new(),
            layout_engine: TextLayoutEngine::new(),
            glyph_cache: LruCache::new(NonZeroUsize::new(GLYPH_CACHE_CAPACITY).unwrap()),
            color_glyph_cache: LruCache::new(
                NonZeroUsize::new(COLOR_GLYPH_CACHE_CAPACITY).unwrap(),
            ),
        }
    }

    /// Get the shared font registry
    ///
    /// This can be used to share the registry with other components
    /// like text measurement.
    pub fn font_registry(&self) -> Arc<std::sync::Mutex<FontRegistry>> {
        self.font_registry.clone()
    }

    /// Set the default font
    pub fn set_default_font(&mut self, font: FontFace) {
        self.default_font = Some(font);
    }

    /// Load and set the default font from file
    pub fn load_default_font(&mut self, path: &std::path::Path) -> Result<()> {
        let font = FontFace::from_file(path)?;
        self.default_font = Some(font);
        Ok(())
    }

    /// Load and set the default font from data
    pub fn load_default_font_data(&mut self, data: Vec<u8>) -> Result<()> {
        let font = FontFace::from_data(data)?;
        self.default_font = Some(font);
        Ok(())
    }

    /// Load font data into the registry (used by the rendering system)
    ///
    /// This adds fonts to the registry where they can be found by name
    /// during text rendering. Returns the number of font faces loaded.
    pub fn load_font_data_to_registry(&mut self, data: Vec<u8>) -> usize {
        let mut registry = self.font_registry.lock().unwrap();
        registry.load_font_data(data)
    }

    /// Get the glyph atlas (grayscale)
    pub fn atlas(&self) -> &GlyphAtlas {
        &self.atlas
    }

    /// Get mutable atlas (for GPU upload checking)
    pub fn atlas_mut(&mut self) -> &mut GlyphAtlas {
        &mut self.atlas
    }

    /// Get the color glyph atlas (RGBA for emoji)
    pub fn color_atlas(&self) -> &ColorGlyphAtlas {
        &self.color_atlas
    }

    /// Get mutable color atlas
    pub fn color_atlas_mut(&mut self) -> &mut ColorGlyphAtlas {
        &mut self.color_atlas
    }

    /// Check if atlas needs GPU upload
    pub fn atlas_is_dirty(&self) -> bool {
        self.atlas.is_dirty()
    }

    /// Check if color atlas needs GPU upload
    pub fn color_atlas_is_dirty(&self) -> bool {
        self.color_atlas.is_dirty()
    }

    /// Mark atlas as clean after GPU upload
    pub fn mark_atlas_clean(&mut self) {
        self.atlas.mark_clean();
    }

    /// Mark color atlas as clean after GPU upload
    pub fn mark_color_atlas_clean(&mut self) {
        self.color_atlas.mark_clean();
    }

    /// Get atlas pixel data for GPU upload (grayscale)
    pub fn atlas_pixels(&self) -> &[u8] {
        self.atlas.pixels()
    }

    /// Get color atlas pixel data for GPU upload (RGBA)
    pub fn color_atlas_pixels(&self) -> &[u8] {
        self.color_atlas.pixels()
    }

    /// Get atlas dimensions
    pub fn atlas_dimensions(&self) -> (u32, u32) {
        self.atlas.dimensions()
    }

    /// Get color atlas dimensions
    pub fn color_atlas_dimensions(&self) -> (u32, u32) {
        self.color_atlas.dimensions()
    }

    /// Prepare text for rendering, rasterizing glyphs as needed
    pub fn prepare_text(
        &mut self,
        text: &str,
        font_size: f32,
        color: [f32; 4],
        options: &LayoutOptions,
    ) -> Result<PreparedText> {
        self.prepare_text_internal(
            text,
            font_size,
            color,
            options,
            None,
            GenericFont::System,
            400,
            false,
        )
    }

    /// Prepare text for rendering with a specific font family
    ///
    /// # Arguments
    /// * `text` - The text to render
    /// * `font_size` - Font size in pixels
    /// * `color` - Text color (RGBA, 0.0-1.0)
    /// * `options` - Layout options
    /// * `font_name` - Optional font name (e.g., "Fira Code", "Inter")
    /// * `generic` - Generic font fallback category
    pub fn prepare_text_with_font(
        &mut self,
        text: &str,
        font_size: f32,
        color: [f32; 4],
        options: &LayoutOptions,
        font_name: Option<&str>,
        generic: GenericFont,
    ) -> Result<PreparedText> {
        self.prepare_text_internal(
            text, font_size, color, options, font_name, generic, 400, false,
        )
    }

    /// Prepare text for rendering with a specific font family, weight, and style
    ///
    /// # Arguments
    /// * `text` - The text to render
    /// * `font_size` - Font size in pixels
    /// * `color` - Text color (RGBA, 0.0-1.0)
    /// * `options` - Layout options
    /// * `font_name` - Optional font name (e.g., "Fira Code", "Inter")
    /// * `generic` - Generic font fallback category
    /// * `weight` - Font weight (100-900, where 400 is normal, 700 is bold)
    /// * `italic` - Whether to use italic variant
    pub fn prepare_text_with_style(
        &mut self,
        text: &str,
        font_size: f32,
        color: [f32; 4],
        options: &LayoutOptions,
        font_name: Option<&str>,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> Result<PreparedText> {
        self.prepare_text_internal(
            text, font_size, color, options, font_name, generic, weight, italic,
        )
    }

    /// Internal method for preparing text with optional font family
    fn prepare_text_internal(
        &mut self,
        text: &str,
        font_size: f32,
        color: [f32; 4],
        options: &LayoutOptions,
        font_name: Option<&str>,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> Result<PreparedText> {
        // Resolve the font to use
        let font = self.resolve_font_with_style(font_name, generic, weight, italic)?;
        let font_id = self.font_id_with_style(font_name, generic, weight, italic);

        // Get font metrics for the PreparedText result
        let (ascender, descender) = {
            let metrics = font.metrics();
            (
                metrics.ascender_px(font_size),
                metrics.descender_px(font_size),
            )
        };

        // Layout the text (positions are based on the primary font only)
        let layout = self.layout_engine.layout(text, &font, font_size, options);

        // Convert to GPU glyph instances
        let mut glyphs = Vec::with_capacity(layout.glyph_count());
        let atlas_dims = self.atlas.dimensions();
        let color_atlas_dims = self.color_atlas.dimensions();
        let (glyph_infos, corrected_width) = self.build_glyph_infos_with_fallback(
            &layout, &font, font_id, font_size, options, weight, italic,
        )?;

        // Second pass: build glyph instances
        for glyph_data in &glyph_infos {
            let data = match glyph_data {
                Some(d) => d,
                None => continue,
            };

            // Skip glyphs with no bitmap (empty glyphs)
            if data.info.region.width == 0 || data.info.region.height == 0 {
                continue;
            }

            // Calculate screen position
            // positioned.x is the pen position from the shaper (includes advance)
            // bearing_x is the offset from pen position to the glyph's left edge
            let x = data.positioned.x + data.info.bearing_x as f32;
            let y = data.positioned.y - data.info.bearing_y as f32;
            let w = data.info.region.width as f32;
            let h = data.info.region.height as f32;

            // Get UV coordinates from the appropriate atlas
            let uv = if data.is_color {
                data.info
                    .region
                    .uv_bounds(color_atlas_dims.0, color_atlas_dims.1)
            } else {
                data.info.region.uv_bounds(atlas_dims.0, atlas_dims.1)
            };

            glyphs.push(GlyphInstance {
                bounds: [x, y, w, h],
                uv_bounds: uv,
                color,
                is_color: data.is_color,
            });
        }

        Ok(PreparedText {
            glyphs,
            width: corrected_width,
            height: layout.height,
            ascender,
            descender,
        })
    }

    /// Prepare styled text with multiple color spans
    ///
    /// This renders text as a single unit but applies different colors to different ranges.
    /// Unlike creating separate text elements, this ensures proper character spacing.
    pub fn prepare_styled_text(
        &mut self,
        text: &str,
        font_size: f32,
        default_color: [f32; 4],
        color_spans: &[ColorSpan],
        options: &LayoutOptions,
        font_name: Option<&str>,
        generic: GenericFont,
    ) -> Result<PreparedText> {
        // Resolve the font to use
        let font = self.resolve_font(font_name, generic)?;
        let font_id = self.font_id(font_name, generic);
        let requested_weight = font.weight().to_number();
        let requested_italic = matches!(font.style(), FontStyle::Italic | FontStyle::Oblique);

        // Get font metrics
        let (ascender, descender) = {
            let metrics = font.metrics();
            (
                metrics.ascender_px(font_size),
                metrics.descender_px(font_size),
            )
        };

        // Layout the text (this gives us proper positions from HarfBuzz)
        let layout = self.layout_engine.layout(text, &font, font_size, options);

        // Build a map of byte position to color
        // For each character, find which span it belongs to
        let get_color_for_byte_pos = |byte_pos: usize| -> [f32; 4] {
            for span in color_spans {
                if byte_pos >= span.start && byte_pos < span.end {
                    return span.color;
                }
            }
            default_color
        };

        // Convert to GPU glyph instances
        let mut glyphs = Vec::with_capacity(layout.glyph_count());
        let atlas_dims = self.atlas.dimensions();
        let color_atlas_dims = self.color_atlas.dimensions();

        let (glyph_infos, corrected_width) = self.build_glyph_infos_with_fallback(
            &layout,
            &font,
            font_id,
            font_size,
            options,
            requested_weight,
            requested_italic,
        )?;

        // Second pass: build glyph instances with per-glyph colors.
        //
        // Important: glyph index is NOT equivalent to character index for complex scripts,
        // ligatures, or emoji sequences. Use HarfBuzz cluster (byte index) to map back to
        // the originating text span.
        for glyph_data in &glyph_infos {
            let data = match glyph_data {
                Some(d) => d,
                None => continue,
            };

            if data.info.region.width == 0 || data.info.region.height == 0 {
                continue;
            }

            // Use HarfBuzz cluster byte index to determine color span.
            let color = get_color_for_byte_pos(data.positioned.cluster as usize);

            // positioned.x is the pen position from the shaper
            // bearing_x is the offset from pen position to the glyph's left edge
            let x = data.positioned.x + data.info.bearing_x as f32;
            let y = data.positioned.y - data.info.bearing_y as f32;
            let w = data.info.region.width as f32;
            let h = data.info.region.height as f32;

            let uv = if data.is_color {
                data.info
                    .region
                    .uv_bounds(color_atlas_dims.0, color_atlas_dims.1)
            } else {
                data.info.region.uv_bounds(atlas_dims.0, atlas_dims.1)
            };

            glyphs.push(GlyphInstance {
                bounds: [x, y, w, h],
                uv_bounds: uv,
                color,
                is_color: data.is_color,
            });
        }

        Ok(PreparedText {
            glyphs,
            width: corrected_width,
            height: layout.height,
            ascender,
            descender,
        })
    }

    /// Resolve font by name or generic category, with fallback to default
    /// Uses only cached fonts - fonts should be preloaded at app startup
    fn resolve_font(
        &mut self,
        font_name: Option<&str>,
        generic: GenericFont,
    ) -> Result<Arc<FontFace>> {
        self.resolve_font_with_style(font_name, generic, 400, false)
    }

    /// Resolve font by name or generic category with specific weight and style
    /// Loads fonts on demand if not cached
    fn resolve_font_with_style(
        &mut self,
        font_name: Option<&str>,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> Result<Arc<FontFace>> {
        let mut registry = self.font_registry.lock().unwrap();

        // First try cache lookup
        if let Some(font) = registry.get_for_render_with_style(font_name, generic, weight, italic) {
            return Ok(font);
        }

        // Try loading the font with style on demand
        if let Some(name) = font_name {
            if let Ok(font) = registry.load_font_with_style(name, weight, italic) {
                return Ok(font);
            }
        }

        // Try loading generic font with style
        if let Ok(font) = registry.load_generic_with_style(generic, weight, italic) {
            return Ok(font);
        }

        // If styled font not found, fall back to normal style
        if weight != 400 || italic {
            if let Some(font) = registry.get_for_render_with_style(font_name, generic, 400, false) {
                return Ok(font);
            }
            // Try loading normal style
            if let Ok(font) = registry.load_generic_with_style(generic, 400, false) {
                return Ok(font);
            }
        }

        // Ultimate fallback to SansSerif normal
        if let Some(font) = registry.get_cached_generic(GenericFont::SansSerif) {
            return Ok(font);
        }
        if let Ok(font) = registry.load_generic(GenericFont::SansSerif) {
            return Ok(font);
        }

        Err(TextError::FontLoadError("No fonts available".to_string()))
    }

    /// Preload fonts that your app uses (call at startup)
    pub fn preload_fonts(&mut self, names: &[&str]) {
        let mut registry = self.font_registry.lock().unwrap();
        registry.preload_fonts(names);
    }

    /// Preload fonts with specific weights and styles
    pub fn preload_fonts_with_styles(&mut self, specs: &[(&str, u16, bool)]) {
        let mut registry = self.font_registry.lock().unwrap();
        for (name, weight, italic) in specs {
            let _ = registry.load_font_with_style(name, *weight, *italic);
        }
    }

    /// Preload generic font variants (weights and italic)
    pub fn preload_generic_styles(&mut self, generic: GenericFont, weights: &[u16], italic: bool) {
        let mut registry = self.font_registry.lock().unwrap();
        for weight in weights {
            let _ = registry.load_generic_with_style(generic, *weight, italic);
        }
    }

    /// Generate a unique font ID for cache keys
    fn font_id(&self, font_name: Option<&str>, generic: GenericFont) -> u32 {
        self.font_id_with_style(font_name, generic, 400, false)
    }

    /// Generate a unique font ID for cache keys with style
    fn font_id_with_style(
        &self,
        font_name: Option<&str>,
        generic: GenericFont,
        weight: u16,
        italic: bool,
    ) -> u32 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        font_name.hash(&mut hasher);
        generic.hash(&mut hasher);
        weight.hash(&mut hasher);
        italic.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Rasterize a glyph for a specific font
    fn rasterize_glyph_for_font(
        &mut self,
        font: &FontFace,
        font_id: u32,
        glyph_id: u16,
        font_size: f32,
    ) -> Result<GlyphInfo> {
        // Quantize font size for cache key (0.5px granularity)
        let size_key = (font_size * 2.0).round() as u16;
        let cache_key = (font_id, glyph_id, size_key);

        // Check cache first (LruCache::get promotes to most-recently-used)
        if let Some(info) = self.glyph_cache.get(&cache_key) {
            return Ok(*info);
        }

        // Rasterize the glyph
        let rasterized = self.rasterizer.rasterize(font, glyph_id, font_size)?;

        // Handle empty glyphs (like space)
        if rasterized.width == 0 || rasterized.height == 0 {
            let info = GlyphInfo {
                region: crate::atlas::AtlasRegion {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                bearing_x: rasterized.bearing_x,
                bearing_y: rasterized.bearing_y,
                advance: rasterized.advance,
                font_size,
            };
            // LruCache::put evicts oldest entry if at capacity
            self.glyph_cache.put(cache_key, info);
            return Ok(info);
        }

        // Insert into atlas
        let info = self.atlas.insert_glyph(
            font_id,
            glyph_id,
            font_size,
            rasterized.width,
            rasterized.height,
            rasterized.bearing_x,
            rasterized.bearing_y,
            rasterized.advance,
            &rasterized.bitmap,
        )?;

        self.glyph_cache.put(cache_key, info);
        Ok(info)
    }

    /// Rasterize a color glyph (emoji) for a specific font
    fn rasterize_color_glyph_for_font(
        &mut self,
        font: &FontFace,
        font_id: u32,
        glyph_id: u16,
        font_size: f32,
    ) -> Result<GlyphInfo> {
        // Quantize font size for cache key (0.5px granularity)
        let size_key = (font_size * 2.0).round() as u16;
        let cache_key = (font_id, glyph_id, size_key);

        // Check color cache first (LruCache::get promotes to most-recently-used)
        if let Some(info) = self.color_glyph_cache.get(&cache_key) {
            return Ok(*info);
        }

        // Rasterize the glyph as color (RGBA)
        let rasterized = self.rasterizer.rasterize_color(font, glyph_id, font_size)?;

        // Handle empty glyphs
        if rasterized.width == 0 || rasterized.height == 0 {
            let info = GlyphInfo {
                region: crate::atlas::AtlasRegion {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                bearing_x: rasterized.bearing_x,
                bearing_y: rasterized.bearing_y,
                advance: rasterized.advance,
                font_size,
            };
            // LruCache::put evicts oldest entry if at capacity
            self.color_glyph_cache.put(cache_key, info);
            return Ok(info);
        }

        // Insert into color atlas
        let info = self.color_atlas.insert_glyph(
            font_id,
            glyph_id,
            font_size,
            rasterized.width,
            rasterized.height,
            rasterized.bearing_x,
            rasterized.bearing_y,
            rasterized.advance,
            &rasterized.bitmap,
        )?;

        self.color_glyph_cache.put(cache_key, info);
        Ok(info)
    }

    /// Legacy method for backward compatibility - uses system font from registry
    #[allow(dead_code)]
    fn rasterize_glyph_if_needed(&mut self, glyph_id: u16, font_size: f32) -> Result<GlyphInfo> {
        let font = {
            let mut registry = self.font_registry.lock().unwrap();
            registry.load_generic(GenericFont::SansSerif)?
        };
        self.rasterize_glyph_for_font(&font, 0, glyph_id, font_size)
    }

    /// Clear the glyph cache and atlas
    pub fn clear(&mut self) {
        self.atlas.clear();
        self.color_atlas.clear();
        self.glyph_cache.clear();
        self.color_glyph_cache.clear();
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}
