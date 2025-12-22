//! Render context for blinc_app
//!
//! Wraps the GPU rendering pipeline with a clean API.

use blinc_core::Rect;
use blinc_gpu::{GpuGlyph, GpuPaintContext, GpuRenderer, TextRenderingContext};
use blinc_layout::prelude::*;
use blinc_layout::renderer::ElementType;
use blinc_svg::SvgDocument;
use blinc_text::TextAnchor;
use std::sync::Arc;

use crate::error::Result;

/// Internal render context that manages GPU resources and rendering
pub struct RenderContext {
    renderer: GpuRenderer,
    text_ctx: TextRenderingContext,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    sample_count: u32,
}

impl RenderContext {
    /// Create a new render context
    pub(crate) fn new(
        renderer: GpuRenderer,
        text_ctx: TextRenderingContext,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        sample_count: u32,
    ) -> Self {
        Self {
            renderer,
            text_ctx,
            device,
            queue,
            sample_count,
        }
    }

    /// Render a layout tree to a texture view
    ///
    /// Handles everything automatically - glass, text, SVG, MSAA.
    pub fn render_tree(
        &mut self,
        tree: &RenderTree,
        width: u32,
        height: u32,
        target: &wgpu::TextureView,
    ) -> Result<()> {
        // Create paint contexts for each layer
        let mut bg_ctx = GpuPaintContext::new(width as f32, height as f32);
        let mut fg_ctx = GpuPaintContext::new(width as f32, height as f32);

        // Render layout layers
        tree.render_to_layer(&mut bg_ctx, RenderLayer::Background);
        tree.render_to_layer(&mut bg_ctx, RenderLayer::Glass);
        tree.render_to_layer(&mut fg_ctx, RenderLayer::Foreground);

        // Collect text and SVG elements
        let (texts, svgs) = self.collect_render_elements(tree);

        // Prepare text glyphs
        let mut all_glyphs = Vec::new();
        for (content, x, y, _w, h, font_size, color) in &texts {
            if let Ok(glyphs) = self.text_ctx.prepare_text_with_anchor(
                content,
                *x,
                *y + *h / 2.0,
                *font_size,
                *color,
                TextAnchor::Center,
            ) {
                all_glyphs.extend(glyphs);
            }
        }

        // Render SVGs to foreground context
        for (source, x, y, w, h) in &svgs {
            if let Ok(doc) = SvgDocument::from_str(source) {
                doc.render_fit(&mut fg_ctx, Rect::new(*x, *y, *w, *h));
            }
        }

        // Take batches
        let bg_batch = bg_ctx.take_batch();
        let fg_batch = fg_ctx.take_batch();

        self.renderer.resize(width, height);

        // Render background
        self.renderer
            .render_with_clear(target, &bg_batch, [1.0, 1.0, 1.0, 1.0]);

        // Render glass if present
        if bg_batch.glass_count() > 0 {
            // Create backdrop texture for glass sampling
            let backdrop = self.create_backdrop_texture(width, height);
            let backdrop_view = backdrop.create_view(&wgpu::TextureViewDescriptor::default());

            // Copy current content to backdrop
            self.copy_texture_to_texture(target, &backdrop, width, height);

            // Render glass with backdrop blur
            self.renderer.render_glass(target, &backdrop_view, &bg_batch);
        }

        // Render foreground on top
        if fg_batch.primitive_count() > 0 {
            self.renderer
                .render_overlay_msaa(target, &fg_batch, self.sample_count);
        }

        // Render text
        if !all_glyphs.is_empty() {
            self.render_text(target, &all_glyphs);
        }

        Ok(())
    }

    /// Create a backdrop texture for glass effects
    fn create_backdrop_texture(&self, width: u32, height: u32) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glass Backdrop"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    /// Copy texture contents
    fn copy_texture_to_texture(
        &self,
        _src_view: &wgpu::TextureView,
        _dst: &wgpu::Texture,
        _width: u32,
        _height: u32,
    ) {
        // Note: wgpu doesn't allow copying from a view directly.
        // In a real implementation, we'd need the source texture handle.
        // For now, glass will sample from a potentially stale backdrop.
        // This would be fixed by tracking textures alongside views.
    }

    /// Render text glyphs
    fn render_text(&mut self, target: &wgpu::TextureView, glyphs: &[GpuGlyph]) {
        if let Some(atlas_view) = self.text_ctx.atlas_view() {
            self.renderer
                .render_text(target, glyphs, atlas_view, self.text_ctx.sampler());
        }
    }

    /// Collect text and SVG elements from the render tree
    fn collect_render_elements(
        &self,
        tree: &RenderTree,
    ) -> (
        Vec<(String, f32, f32, f32, f32, f32, [f32; 4])>,
        Vec<(String, f32, f32, f32, f32)>,
    ) {
        let mut texts = Vec::new();
        let mut svgs = Vec::new();

        if let Some(root) = tree.root() {
            self.collect_elements_recursive(tree, root, (0.0, 0.0), &mut texts, &mut svgs);
        }

        (texts, svgs)
    }

    fn collect_elements_recursive(
        &self,
        tree: &RenderTree,
        node: LayoutNodeId,
        parent_offset: (f32, f32),
        texts: &mut Vec<(String, f32, f32, f32, f32, f32, [f32; 4])>,
        svgs: &mut Vec<(String, f32, f32, f32, f32)>,
    ) {
        let Some(bounds) = tree.layout().get_bounds(node, parent_offset) else {
            return;
        };

        let abs_x = bounds.x;
        let abs_y = bounds.y;

        if let Some(render_node) = tree.get_render_node(node) {
            match &render_node.element_type {
                ElementType::Text(text_data) => {
                    texts.push((
                        text_data.content.clone(),
                        abs_x,
                        abs_y,
                        bounds.width,
                        bounds.height,
                        text_data.font_size,
                        text_data.color,
                    ));
                }
                ElementType::Svg(svg_data) => {
                    svgs.push((
                        svg_data.source.clone(),
                        abs_x,
                        abs_y,
                        bounds.width,
                        bounds.height,
                    ));
                }
                ElementType::Div => {}
            }
        }

        let new_offset = (abs_x, abs_y);
        for child_id in tree.layout().children(node) {
            self.collect_elements_recursive(tree, child_id, new_offset, texts, svgs);
        }
    }

    /// Get device arc
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    /// Get queue arc
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
}
