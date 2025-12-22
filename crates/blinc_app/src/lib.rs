//! Blinc Application Framework
//!
//! Clean API for building Blinc applications with layout and rendering.
//!
//! # Example
//!
//! ```ignore
//! use blinc_app::prelude::*;
//!
//! fn main() -> Result<()> {
//!     let app = BlincApp::new()?;
//!
//!     let ui = div()
//!         .w(400.0).h(300.0)
//!         .flex_col().gap(4.0).p(4.0)
//!         .child(
//!             div().glass()
//!                 .w_full().h(100.0)
//!                 .rounded(16.0)
//!                 .child(text("Hello Blinc!").size(24.0))
//!         );
//!
//!     app.render(&ui, 400.0, 300.0)?;
//! }
//! ```

mod app;
mod context;
mod error;

#[cfg(test)]
mod tests;

pub use app::{BlincApp, BlincConfig};
pub use context::RenderContext;
pub use error::{BlincError, Result};

// Re-export layout API for convenience
pub use blinc_layout::prelude::*;
pub use blinc_layout::RenderTree;

/// Prelude module - import everything commonly needed
pub mod prelude {
    pub use crate::app::{BlincApp, BlincConfig};
    pub use crate::context::RenderContext;
    pub use crate::error::{BlincError, Result};

    // Layout builders
    pub use blinc_layout::prelude::*;
    pub use blinc_layout::RenderTree;

    // Core types
    pub use blinc_core::{Color, Point, Rect, Size};
}
