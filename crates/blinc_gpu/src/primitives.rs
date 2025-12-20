//! GPU primitive batching

/// A GPU primitive ready for rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuRect {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub corner_radius: [f32; 4],
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub _padding: [f32; 3],
}

/// Batch of GPU primitives
pub struct PrimitiveBatch {
    pub rects: Vec<GpuRect>,
    // TODO: Add other primitive types
}

impl PrimitiveBatch {
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }
}

impl Default for PrimitiveBatch {
    fn default() -> Self {
        Self::new()
    }
}
