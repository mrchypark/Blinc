//! GPU renderer implementation

/// The GPU renderer using wgpu
pub struct GpuRenderer {
    // TODO: wgpu device, queue, pipelines
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}
