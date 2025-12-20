//! Blinc GPU Renderer
//!
//! SDF-based GPU rendering using wgpu.

pub mod primitives;
pub mod renderer;
pub mod shaders;

pub use renderer::GpuRenderer;
