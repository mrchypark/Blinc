//! Blinc Paint/Canvas API
//!
//! A 2D drawing API for custom graphics, similar to HTML Canvas or Skia.
//!
//! # Features
//!
//! - Path drawing (lines, curves, arcs)
//! - Shape primitives (rect, circle, rounded rect)
//! - Fills and strokes with colors, gradients
//! - Text rendering
//! - SDF-based rendering for resolution independence
//! - Clipping and masking

pub mod color;
pub mod context;
pub mod gradient;
pub mod path;
pub mod primitives;

pub use color::Color;
pub use context::PaintContext;
pub use gradient::{Gradient, GradientStop};
pub use path::{Path, PathBuilder};
pub use primitives::*;
