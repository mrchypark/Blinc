//! Headless rendering context for testing.
//!
//! Provides a way to render Blinc UI without a window, useful for:
//! - Unit testing UI components
//! - Integration testing
//! - Visual regression testing
//! - CI/CD pipelines

/// Configuration for headless rendering.
#[derive(Clone, Debug)]
pub struct HeadlessConfig {
    /// Width of the render target in pixels.
    pub width: u32,
    /// Height of the render target in pixels.
    pub height: u32,
    /// Scale factor for HiDPI rendering.
    pub scale_factor: f64,
    /// Whether to enable MSAA.
    pub msaa_samples: u32,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            scale_factor: 1.0,
            msaa_samples: 1,
        }
    }
}

impl HeadlessConfig {
    /// Create a new config with specified dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Set the scale factor.
    pub fn with_scale_factor(mut self, scale: f64) -> Self {
        self.scale_factor = scale;
        self
    }

    /// Enable MSAA with specified sample count.
    pub fn with_msaa(mut self, samples: u32) -> Self {
        self.msaa_samples = samples;
        self
    }
}

/// A headless rendering context for testing.
///
/// This provides a minimal environment for rendering Blinc UI
/// without requiring a window or display.
pub struct HeadlessContext {
    config: HeadlessConfig,
    frame_count: u64,
    // GPU resources will be added when wgpu feature is enabled
    // device: Arc<wgpu::Device>,
    // queue: Arc<wgpu::Queue>,
    // render_texture: wgpu::Texture,
}

impl HeadlessContext {
    /// Create a new headless context with the given configuration.
    ///
    /// This initializes wgpu in headless mode without requiring a surface.
    pub fn new(config: HeadlessConfig) -> Self {
        Self {
            config,
            frame_count: 0,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &HeadlessConfig {
        &self.config
    }

    /// Get the width in logical pixels.
    pub fn width(&self) -> f32 {
        self.config.width as f32 / self.config.scale_factor as f32
    }

    /// Get the height in logical pixels.
    pub fn height(&self) -> f32 {
        self.config.height as f32 / self.config.scale_factor as f32
    }

    /// Get the physical width in pixels.
    pub fn physical_width(&self) -> u32 {
        self.config.width
    }

    /// Get the physical height in pixels.
    pub fn physical_height(&self) -> u32 {
        self.config.height
    }

    /// Get the scale factor.
    pub fn scale_factor(&self) -> f64 {
        self.config.scale_factor
    }

    /// Get the number of frames rendered.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Resize the render target.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        // TODO: Recreate render texture
    }

    /// Advance to the next frame.
    pub fn next_frame(&mut self) {
        self.frame_count += 1;
    }
}

/// Builder for HeadlessContext with fluent API.
#[allow(dead_code)]
pub struct HeadlessContextBuilder {
    config: HeadlessConfig,
}

#[allow(dead_code)]
impl HeadlessContextBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: HeadlessConfig::default(),
        }
    }

    /// Set the dimensions.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    /// Set the scale factor.
    pub fn scale_factor(mut self, scale: f64) -> Self {
        self.config.scale_factor = scale;
        self
    }

    /// Enable MSAA.
    pub fn msaa(mut self, samples: u32) -> Self {
        self.config.msaa_samples = samples;
        self
    }

    /// Build the headless context.
    pub fn build(self) -> HeadlessContext {
        HeadlessContext::new(self.config)
    }
}

impl Default for HeadlessContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_config_defaults() {
        let config = HeadlessConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.scale_factor, 1.0);
    }

    #[test]
    fn test_headless_context_dimensions() {
        let ctx = HeadlessContext::new(HeadlessConfig::new(1920, 1080).with_scale_factor(2.0));

        assert_eq!(ctx.physical_width(), 1920);
        assert_eq!(ctx.physical_height(), 1080);
        assert_eq!(ctx.width(), 960.0);
        assert_eq!(ctx.height(), 540.0);
    }

    #[test]
    fn test_headless_context_builder() {
        let ctx = HeadlessContextBuilder::new()
            .size(1024, 768)
            .scale_factor(1.5)
            .msaa(4)
            .build();

        assert_eq!(ctx.physical_width(), 1024);
        assert_eq!(ctx.physical_height(), 768);
        assert_eq!(ctx.scale_factor(), 1.5);
    }
}
