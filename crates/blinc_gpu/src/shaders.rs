//! GPU shaders for SDF primitives
//!
//! These shaders render:
//! - Rounded rectangles with borders
//! - Circles and ellipses
//! - Gaussian blur shadows
//! - Text glyphs (SDF-based)
//! - Gradients

/// WGSL shader for rounded rectangles
pub const ROUNDED_RECT_SHADER: &str = r#"
// Rounded rectangle SDF shader
// TODO: Implement
"#;

/// WGSL shader for shadows (Gaussian blur via erf)
pub const SHADOW_SHADER: &str = r#"
// Shadow shader using error function approximation
// TODO: Implement
"#;

/// WGSL shader for text rendering
pub const TEXT_SHADER: &str = r#"
// SDF text rendering shader
// TODO: Implement
"#;
