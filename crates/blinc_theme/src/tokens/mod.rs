//! Design tokens for theming
//!
//! Tokens are the atomic values that make up a design system:
//! - Colors
//! - Typography (fonts, sizes, weights)
//! - Spacing (margins, padding)
//! - Opacity
//! - Border radii
//! - Shadows
//! - Animation durations and easings

mod animation;
mod color;
mod opacity;
mod radius;
mod shadow;
mod spacing;
mod typography;

pub use animation::*;
pub use color::*;
pub use opacity::*;
pub use radius::*;
pub use shadow::*;
pub use spacing::*;
pub use typography::*;
