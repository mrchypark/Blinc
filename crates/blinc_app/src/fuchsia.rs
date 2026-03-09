//! Fuchsia application runner
//!
//! Provides the Fuchsia-facing entry point for Blinc applications.
//!
//! Fuchsia is currently a deferred platform target. The runtime entry point is
//! kept so code continues to compile behind feature gates, but execution fails
//! explicitly instead of pretending to succeed with a stubbed runner.
//!
//! # Example
//!
//! ```ignore
//! use blinc_app::prelude::*;
//! use blinc_app::fuchsia::FuchsiaApp;
//!
//! #[no_mangle]
//! fn main() {
//!     FuchsiaApp::run(|ctx| {
//!         div().w(ctx.width).h(ctx.height)
//!             .bg([0.1, 0.1, 0.15, 1.0])
//!             .flex_center()
//!             .child(text("Hello Fuchsia!").size(48.0))
//!     }).unwrap();
//! }
//! ```
//!
//! # Architecture
//!
//! Fuchsia applications integrate with the system through:
//!
//! - **Scenic/Flatland** - Window compositing via Views
//! - **fuchsia-async** - Async executor for event handling
//! - **FIDL** - IPC with system services
//! - **Vulkan** - GPU rendering via ImagePipe2
//!
//! # Building
//!
//! Requires the Fuchsia SDK and target:
//!
//! ```bash
//! rustup target add x86_64-unknown-fuchsia
//! cargo build --target x86_64-unknown-fuchsia --features fuchsia
//! ```

use blinc_layout::prelude::*;
use blinc_platform_fuchsia::FuchsiaPlatform;

use crate::error::{BlincError, Result};
use crate::windowed::WindowedContext;

/// Fuchsia application runner
///
/// Provides the Fuchsia entry point for Blinc applications.
pub struct FuchsiaApp;

impl FuchsiaApp {
    /// Run a Fuchsia Blinc application
    ///
    /// This is the main entry point for Fuchsia applications.
    ///
    /// Fuchsia execution is not currently implemented in-tree. Calling this on
    /// Fuchsia returns an explicit unsupported-platform error instead of silently
    /// succeeding with a stubbed runtime.
    ///
    /// # Arguments
    ///
    /// * `ui_builder` - Function that builds the UI tree given the window context
    ///
    /// # Example
    ///
    /// ```ignore
    /// FuchsiaApp::run(|ctx| {
    ///     div()
    ///         .w(ctx.width).h(ctx.height)
    ///         .bg([0.1, 0.1, 0.15, 1.0])
    ///         .flex_center()
    ///         .child(text("Hello Fuchsia!").size(32.0))
    /// })
    /// ```
    #[cfg(target_os = "fuchsia")]
    pub fn run<F, E>(mut ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder + 'static,
    {
        // Initialize logging
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        tracing::info!("FuchsiaApp::run starting");
        tracing::warn!("Fuchsia platform support is currently unsupported in-tree");
        tracing::warn!(
            "Full Fuchsia integration requires Scenic/Flatland, input, and renderer wiring"
        );
        let _ = &mut ui_builder;
        Err(BlincError::PlatformUnsupported(
            "Fuchsia runtime is currently unsupported in-tree".to_string(),
        ))
    }

    /// Placeholder for non-Fuchsia builds
    #[cfg(not(target_os = "fuchsia"))]
    pub fn run<F, E>(_ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder + 'static,
    {
        Err(BlincError::PlatformUnsupported(
            "Fuchsia apps can only run on Fuchsia OS".to_string(),
        ))
    }

    /// Get the system font paths for Fuchsia
    pub fn system_font_paths() -> &'static [&'static str] {
        FuchsiaPlatform::system_font_paths()
    }
}
