//! Image texture management for GPU rendering
//!
//! Manages GPU textures for images and provides rendering support.

use std::{borrow::Cow, sync::Arc};
use wgpu::util::DeviceExt;

/// A GPU image texture ready for rendering
pub struct GpuImage {
    /// The GPU texture
    texture: wgpu::Texture,
    /// Texture view for sampling
    view: wgpu::TextureView,
    /// Image width
    width: u32,
    /// Image height
    height: u32,
}

impl GpuImage {
    /// Create an empty GPU image (uninitialized contents)
    pub fn empty(device: &wgpu::Device, width: u32, height: u32, label: Option<&str>) -> Self {
        let max_dim = device.limits().max_texture_dimension_2d;
        let width = width.clamp(1, max_dim);
        let height = height.clamp(1, max_dim);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
        }
    }

    /// Create a GPU image from RGBA pixel data
    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pixels: &[u8],
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            pixels,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
        }
    }

    /// Get the texture view for binding
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Get image dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get image width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get image height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the underlying texture
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Write RGBA pixels into a sub-rect of this image
    pub fn write_rgba_sub_rect(
        &self,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) {
        if width == 0 || height == 0 {
            return;
        }

        let max_write_width = self.width.saturating_sub(x);
        let max_write_height = self.height.saturating_sub(y);
        let width = width.min(max_write_width);
        let height = height.min(max_write_height);
        if width == 0 || height == 0 {
            return;
        }

        let bytes_per_pixel = 4usize;
        let width_usize = width as usize;
        let height_usize = height as usize;
        let row_bytes = match width_usize.checked_mul(bytes_per_pixel) {
            Some(v) => v,
            None => return,
        };
        let required_len = match row_bytes.checked_mul(height_usize) {
            Some(v) => v,
            None => return,
        };
        if pixels.len() < required_len {
            return;
        }

        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_row_bytes = match row_bytes
            .checked_add(align - 1)
            .map(|v| (v / align) * align)
        {
            Some(v) => v,
            None => return,
        };
        let padded_total = match padded_row_bytes.checked_mul(height_usize) {
            Some(v) => v,
            None => return,
        };
        let padded_row_bytes_u32 = match u32::try_from(padded_row_bytes) {
            Ok(v) => v,
            Err(_) => return,
        };

        let data: Cow<'_, [u8]> = if padded_row_bytes == row_bytes {
            Cow::Borrowed(&pixels[..required_len])
        } else {
            let mut padded = vec![0u8; padded_total];
            for row in 0..height_usize {
                let src_start = row * row_bytes;
                let dst_start = row * padded_row_bytes;
                padded[dst_start..dst_start + row_bytes]
                    .copy_from_slice(&pixels[src_start..src_start + row_bytes]);
            }
            Cow::Owned(padded)
        };

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes_u32),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// GPU image instance data for batched rendering
///
/// Memory layout (matches shader ImageInstance):
/// - `dst_rect`: `vec4<f32>` (16 bytes) - destination rectangle
/// - `src_uv`: `vec4<f32>` (16 bytes) - source UV coordinates
/// - `tint`: `vec4<f32>` (16 bytes) - tint color
/// - `params`: `vec4<f32>` (16 bytes) - border_radius, opacity, padding, padding
/// - `clip_bounds`: `vec4<f32>` (16 bytes) - clip region
/// - `clip_radius`: `vec4<f32>` (16 bytes) - clip corner radii
/// Total: 96 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuImageInstance {
    /// Destination rectangle (x, y, width, height) in screen pixels
    pub dst_rect: [f32; 4],
    /// Source UV rectangle (u_min, v_min, u_max, v_max) normalized 0-1
    pub src_uv: [f32; 4],
    /// Tint color (RGBA)
    pub tint: [f32; 4],
    /// Parameters: (border_radius, opacity, padding, padding)
    pub params: [f32; 4],
    /// Clip bounds (x, y, width, height) - set to large negative x for no clip
    pub clip_bounds: [f32; 4],
    /// Clip corner radii (top-left, top-right, bottom-right, bottom-left)
    pub clip_radius: [f32; 4],
}

impl Default for GpuImageInstance {
    fn default() -> Self {
        Self {
            dst_rect: [0.0, 0.0, 100.0, 100.0],
            src_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0, 1.0, 1.0, 1.0],
            params: [0.0, 1.0, 0.0, 0.0], // border_radius=0, opacity=1
            // Default: no clip (large negative value disables clipping)
            clip_bounds: [-10000.0, -10000.0, 100000.0, 100000.0],
            clip_radius: [0.0; 4],
        }
    }
}

impl GpuImageInstance {
    /// Create a new image instance with no transformations
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            dst_rect: [x, y, width, height],
            ..Default::default()
        }
    }

    /// Set the source UV coordinates for cropping
    pub fn with_src_uv(mut self, u_min: f32, v_min: f32, u_max: f32, v_max: f32) -> Self {
        self.src_uv = [u_min, v_min, u_max, v_max];
        self
    }

    /// Set a tint color
    pub fn with_tint(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.tint = [r, g, b, a];
        self
    }

    /// Set border radius for rounded corners
    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.params[0] = radius;
        self
    }

    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.params[1] = opacity;
        self
    }

    /// Set rectangular clip region
    pub fn with_clip_rect(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.clip_bounds = [x, y, width, height];
        self.clip_radius = [0.0; 4];
        self
    }

    /// Set rounded rectangular clip region with uniform radius
    pub fn with_clip_rounded_rect(
        mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    ) -> Self {
        self.clip_bounds = [x, y, width, height];
        self.clip_radius = [radius; 4];
        self
    }

    /// Set rounded rectangular clip region with per-corner radii
    pub fn with_clip_rounded_rect_corners(
        mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    ) -> Self {
        self.clip_bounds = [x, y, width, height];
        self.clip_radius = [tl, tr, br, bl];
        self
    }

    /// Clear clip region (no clipping)
    pub fn with_no_clip(mut self) -> Self {
        self.clip_bounds = [-10000.0, -10000.0, 100000.0, 100000.0];
        self.clip_radius = [0.0; 4];
        self
    }

    /// Get border radius
    pub fn border_radius(&self) -> f32 {
        self.params[0]
    }

    /// Get opacity
    pub fn opacity(&self) -> f32 {
        self.params[1]
    }
}

/// Image rendering context
pub struct ImageRenderingContext {
    /// Device reference
    device: Arc<wgpu::Device>,
    /// Queue reference
    queue: Arc<wgpu::Queue>,
    /// Image sampler (linear filtering)
    sampler_linear: wgpu::Sampler,
    /// Image sampler (nearest filtering, for pixel art)
    sampler_nearest: wgpu::Sampler,
}

impl ImageRenderingContext {
    /// Create a new image rendering context
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler (Linear)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler (Nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            device,
            queue,
            sampler_linear,
            sampler_nearest,
        }
    }

    /// Create a GPU image from RGBA data
    pub fn create_image(&self, pixels: &[u8], width: u32, height: u32) -> GpuImage {
        GpuImage::from_rgba(&self.device, &self.queue, pixels, width, height, None)
    }

    /// Create a GPU image with a label
    pub fn create_image_labeled(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        label: &str,
    ) -> GpuImage {
        GpuImage::from_rgba(
            &self.device,
            &self.queue,
            pixels,
            width,
            height,
            Some(label),
        )
    }

    /// Create an empty GPU image
    pub fn create_empty_image(&self, width: u32, height: u32) -> GpuImage {
        GpuImage::empty(&self.device, width, height, None)
    }

    /// Create an empty GPU image with a label
    pub fn create_empty_image_labeled(&self, width: u32, height: u32, label: &str) -> GpuImage {
        GpuImage::empty(&self.device, width, height, Some(label))
    }

    /// Get the linear sampler
    pub fn sampler_linear(&self) -> &wgpu::Sampler {
        &self.sampler_linear
    }

    /// Get the nearest sampler
    pub fn sampler_nearest(&self) -> &wgpu::Sampler {
        &self.sampler_nearest
    }

    /// Get the device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
