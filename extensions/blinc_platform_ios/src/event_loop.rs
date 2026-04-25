//! iOS event loop implementation
//!
//! Uses CADisplayLink for vsync-aligned rendering and RunLoop for event handling.

use crate::window::IOSWindow;
use blinc_platform::{ControlFlow, Event, EventLoop, PlatformError};

#[cfg(target_os = "ios")]
use crate::app::current_window;

#[cfg(target_os = "ios")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "ios")]
use std::sync::Arc;

#[cfg(target_os = "ios")]
use tracing::info;

/// Wake proxy for iOS event loop
///
/// Use this to request a redraw from a background animation thread.
#[cfg(target_os = "ios")]
#[derive(Clone)]
pub struct IOSWakeProxy {
    /// Flag indicating a wake was requested
    wake_requested: Arc<AtomicBool>,
}

#[cfg(target_os = "ios")]
impl Default for IOSWakeProxy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "ios")]
impl IOSWakeProxy {
    /// Create a new wake proxy
    pub fn new() -> Self {
        Self {
            wake_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Wake up the event loop, causing it to process events and potentially redraw
    pub fn wake(&self) {
        self.wake_requested.store(true, Ordering::SeqCst);
        // On iOS, we rely on CADisplayLink which runs at vsync
        // The wake_requested flag will be checked on next frame
    }

    /// Check if a wake was requested and clear the flag
    pub fn take_wake_request(&self) -> bool {
        self.wake_requested.swap(false, Ordering::SeqCst)
    }
}

/// Placeholder wake proxy for non-iOS builds
#[cfg(not(target_os = "ios"))]
#[derive(Clone, Default)]
pub struct IOSWakeProxy;

#[cfg(not(target_os = "ios"))]
impl IOSWakeProxy {
    /// Create a placeholder wake proxy
    pub fn new() -> Self {
        Self
    }

    /// No-op wake for non-iOS
    pub fn wake(&self) {}

    /// Always returns false on non-iOS
    pub fn take_wake_request(&self) -> bool {
        false
    }
}

/// iOS event loop using CADisplayLink
#[cfg(target_os = "ios")]
pub struct IOSEventLoop {
    /// Wake proxy for animation thread
    wake_proxy: IOSWakeProxy,
}

#[cfg(target_os = "ios")]
impl Default for IOSEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "ios")]
impl IOSEventLoop {
    /// Create a new iOS event loop
    pub fn new() -> Self {
        Self {
            wake_proxy: IOSWakeProxy::new(),
        }
    }

    /// Get a wake proxy that can be used to wake up the event loop from another thread
    pub fn wake_proxy(&self) -> IOSWakeProxy {
        self.wake_proxy.clone()
    }
}

#[cfg(target_os = "ios")]
impl EventLoop for IOSEventLoop {
    type Window = IOSWindow;

    fn run<F>(self, _handler: F) -> Result<(), PlatformError>
    where
        F: FnMut(Event, &Self::Window) -> ControlFlow + 'static,
    {
        info!("iOS event loop is UIKit-managed; dispatching initial bridge events");

        // UIKit owns the run loop on iOS. From Rust we can only expose a small
        // integration surface and hand off control back to the host app.
        if let Some(window) = current_window() {
            for event in managed_event_sequence() {
                if matches!(handler(event, &window), ControlFlow::Exit) {
                    break;
                }
            }
        } else {
            warn!("No active UIWindow available for initial iOS bridge events");
        }
        Ok(())
    }
}

/// Placeholder for non-iOS builds
#[cfg(not(target_os = "ios"))]
#[derive(Default)]
pub struct IOSEventLoop {
    _private: (),
}

#[cfg(not(target_os = "ios"))]
impl IOSEventLoop {
    /// Create a placeholder event loop
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Get a placeholder wake proxy
    pub fn wake_proxy(&self) -> IOSWakeProxy {
        IOSWakeProxy
    }
}

#[cfg(not(target_os = "ios"))]
impl EventLoop for IOSEventLoop {
    type Window = IOSWindow;

    fn run<F>(self, _handler: F) -> Result<(), PlatformError>
    where
        F: FnMut(Event, &Self::Window) -> ControlFlow + 'static,
    {
        Err(PlatformError::Unsupported(
            "iOS platform only available on iOS".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::managed_event_sequence;
    use blinc_platform::{Event, LifecycleEvent};

    #[test]
    fn managed_event_sequence_starts_with_resume_then_frame() {
        let events = managed_event_sequence();
        assert!(matches!(
            events[0],
            Event::Lifecycle(LifecycleEvent::Resumed)
        ));
        assert!(matches!(events[1], Event::Frame));
    }
}
