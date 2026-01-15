//! Blinc Fuchsia Platform
//!
//! Scenic compositor integration and Vulkan rendering for Fuchsia OS.
//!
//! This crate implements the `blinc_platform` traits for Fuchsia,
//! providing touch/mouse input, lifecycle management, and window handling
//! via Scenic and FIDL.
//!
//! # Architecture
//!
//! Fuchsia uses a component-based architecture where Blinc integrates through:
//!
//! - **Scenic** for window compositing via Views
//! - **fuchsia-async** for async event handling
//! - **FIDL** for IPC with system services
//! - **Vulkan** for GPU rendering via ImagePipe
//!
//! # Usage
//!
//! ```ignore
//! use blinc_app::fuchsia::FuchsiaApp;
//!
//! fn main() {
//!     FuchsiaApp::run(|ctx| {
//!         div()
//!             .w(ctx.width).h(ctx.height)
//!             .bg([0.1, 0.1, 0.15, 1.0])
//!             .flex_center()
//!             .child(text("Hello Fuchsia!").size(48.0))
//!     }).unwrap();
//! }
//! ```
//!
//! # Building for Fuchsia
//!
//! Requires the Fuchsia SDK and appropriate Rust targets:
//!
//! ```bash
//! rustup target add x86_64-unknown-fuchsia
//! rustup target add aarch64-unknown-fuchsia
//! ```

pub mod app;
pub mod assets;
pub mod event_loop;
pub mod input;
pub mod window;

// Re-export public types
pub use app::FuchsiaPlatform;
pub use assets::FuchsiaAssetLoader;
pub use event_loop::{FuchsiaEventLoop, FuchsiaWakeProxy};
pub use input::{convert_touch, Touch, TouchPhase};
pub use window::FuchsiaWindow;

use blinc_platform::PlatformError;

// Convenience constructor for non-Fuchsia builds
#[cfg(not(target_os = "fuchsia"))]
impl FuchsiaPlatform {
    /// Create a placeholder platform (for cross-compilation checks)
    pub fn with_placeholder() -> Result<Self, PlatformError> {
        Err(PlatformError::Unsupported(
            "Fuchsia platform only available on Fuchsia OS".to_string(),
        ))
    }
}
