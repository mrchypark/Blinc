//! Fuchsia event loop implementation
//!
//! Uses fuchsia-async executor for async event handling.

use blinc_platform::{ControlFlow, Event, EventLoop, PlatformError};

use crate::window::FuchsiaWindow;

#[cfg(target_os = "fuchsia")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "fuchsia")]
use std::sync::Arc;

/// Wake proxy for Fuchsia event loop
///
/// Use this to request a redraw from a background animation thread.
#[derive(Clone)]
pub struct FuchsiaWakeProxy {
    #[cfg(target_os = "fuchsia")]
    wake_requested: Arc<AtomicBool>,
}

impl FuchsiaWakeProxy {
    /// Create a new wake proxy
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "fuchsia")]
            wake_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Wake up the event loop
    pub fn wake(&self) {
        #[cfg(target_os = "fuchsia")]
        {
            self.wake_requested.store(true, Ordering::SeqCst);
            // TODO: Wake the fuchsia-async executor
        }
    }

    /// Check if a wake was requested and clear the flag
    #[cfg(target_os = "fuchsia")]
    pub fn take_wake_request(&self) -> bool {
        self.wake_requested.swap(false, Ordering::SeqCst)
    }
}

impl Default for FuchsiaWakeProxy {
    fn default() -> Self {
        Self::new()
    }
}

/// Fuchsia event loop using fuchsia-async
pub struct FuchsiaEventLoop {
    wake_proxy: FuchsiaWakeProxy,
}

impl FuchsiaEventLoop {
    /// Create a new Fuchsia event loop
    pub fn new() -> Self {
        Self {
            wake_proxy: FuchsiaWakeProxy::new(),
        }
    }

    /// Get a wake proxy for animation threads
    pub fn wake_proxy(&self) -> FuchsiaWakeProxy {
        self.wake_proxy.clone()
    }
}

impl Default for FuchsiaEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLoop for FuchsiaEventLoop {
    type Window = FuchsiaWindow;

    fn run<F>(self, mut _handler: F) -> Result<(), PlatformError>
    where
        F: FnMut(Event, &Self::Window) -> ControlFlow + 'static,
    {
        #[cfg(target_os = "fuchsia")]
        {
            // TODO: Run fuchsia-async executor
            // - Handle FIDL events from Scenic
            // - Handle input from fuchsia.ui.pointer
            // - Schedule frames based on vsync
            tracing::info!("Fuchsia event loop started");
            Ok(())
        }

        #[cfg(not(target_os = "fuchsia"))]
        {
            Err(PlatformError::Unsupported(
                "Fuchsia event loop only available on Fuchsia OS".to_string(),
            ))
        }
    }
}
