//! Blinc Platform Abstraction Layer
//!
//! This crate provides platform-agnostic traits and types for windowing,
//! input handling, and application lifecycle management.
//!
//! # Architecture
//!
//! The platform abstraction is built around three main traits:
//!
//! - [`Platform`] - The top-level platform abstraction
//! - [`Window`] - Window management and properties
//! - [`EventLoop`] - Event handling and application lifecycle
//!
//! # Platform Implementations
//!
//! - `blinc_platform_desktop` - Desktop platforms (macOS, Windows, Linux) using winit
//! - `blinc_platform_android` - Android using NDK
//! - `blinc_platform_ios` - iOS using UIKit (planned)
//!
//! # Example
//!
//! ```ignore
//! use blinc_platform::*;
//! use blinc_platform_desktop::DesktopPlatform;
//!
//! fn main() -> Result<(), PlatformError> {
//!     let platform = DesktopPlatform::new()?;
//!     let event_loop = platform.create_event_loop()?;
//!
//!     event_loop.run(|event, window| {
//!         match event {
//!             Event::Frame => {
//!                 // Render frame
//!             }
//!             Event::Window(WindowEvent::CloseRequested) => {
//!                 return ControlFlow::Exit;
//!             }
//!             _ => {}
//!         }
//!         ControlFlow::Continue
//!     })
//! }
//! ```

pub mod accessibility;
pub mod app;
pub mod assets;
pub mod ble;
pub mod clipboard;
pub mod deep_link;
pub mod environment;
mod error;
mod event;
pub mod haptics;
mod ime;
mod input;
pub mod microphone;
pub mod permissions;
mod platform;
pub mod sensors;
mod window;

// Re-export all public types
pub use environment::{
    LifecycleState, PlatformEnvironmentChanged, PlatformEnvironmentSnapshot, ViewportInsets,
    WindowMetrics,
};
pub use error::{PlatformError, Result};
pub use event::{ControlFlow, Event, EventLoop, LifecycleEvent, WindowEvent};
pub use ime::{
    current_ime_state, set_ime_state, ImeCursorArea, ImeRequest, ImeState, ImeVisibility,
    SelectionRange, TextInputSessionId,
};
pub use input::{
    FocusTraversalIntent, ImeCompositionSelection, ImeCompositionUpdate, InputEvent, Key, KeyState,
    KeyboardEvent, Modifiers, MouseButton, MouseEvent, ScrollPhase, TouchEvent,
};
pub use platform::Platform;
pub use window::{Cursor, Window, WindowConfig, WindowId};

// Re-export commonly used asset types
pub use accessibility::{
    current_accessibility_sync_status, current_platform_accessibility_snapshot,
    mark_accessibility_unsupported, reset_accessibility_runtime_state,
    update_platform_accessibility_snapshot, AccessibilityAction, AccessibilityActionRequest,
    AccessibilityBounds, AccessibilityNode, AccessibilityNodeId, AccessibilityRole,
    AccessibilitySyncStatus, AccessibilityTreeSnapshot,
};
pub use assets::{AssetLoader, AssetPath, FilesystemAssetLoader};
pub use ble::{
    BleBackend, BleBatchSummary, BleClient, BleProbeState, BleRuntimeController, BleScanConfig,
    BleScanResult, BleScanStatus,
};
pub use permissions::{PermissionKind, PermissionStatus};
pub use sensors::{
    NativeBridgePermissionBackend, SensorAccuracy, SensorBackend, SensorBatchSummary, SensorClient,
    SensorConfig, SensorError, SensorFrame, SensorKind, SensorPermissionBackend,
    SensorPermissionService, SensorPermissionState, SensorProbeState, SensorRuntimeController,
    SensorStatus,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::accessibility::{
        current_accessibility_sync_status, current_platform_accessibility_snapshot,
        mark_accessibility_unsupported, reset_accessibility_runtime_state,
        update_platform_accessibility_snapshot, AccessibilityAction, AccessibilityActionRequest,
        AccessibilityBounds, AccessibilityNode, AccessibilityNodeId, AccessibilityRole,
        AccessibilitySyncStatus, AccessibilityTreeSnapshot,
    };
    pub use crate::app;
    pub use crate::assets::{
        asset_exists, asset_url, load_asset, load_asset_string, preload_settled, AssetLoader,
        AssetPath, FilesystemAssetLoader,
    };
    pub use crate::ble::{
        BleBackend, BleBatchSummary, BleClient, BleProbeState, BleRuntimeController, BleScanConfig,
        BleScanResult, BleScanStatus,
    };
    pub use crate::environment::{
        LifecycleState, PlatformEnvironmentChanged, PlatformEnvironmentSnapshot, ViewportInsets,
        WindowMetrics,
    };
    pub use crate::error::{PlatformError, Result};
    pub use crate::event::{ControlFlow, Event, EventLoop, LifecycleEvent, WindowEvent};
    pub use crate::haptics;
    pub use crate::ime::{
        current_ime_state, set_ime_state, ImeCursorArea, ImeRequest, ImeState, ImeVisibility,
        SelectionRange, TextInputSessionId,
    };
    pub use crate::input::{
        FocusTraversalIntent, ImeCompositionSelection, ImeCompositionUpdate, InputEvent, Key,
        KeyState, KeyboardEvent, Modifiers, MouseButton, MouseEvent, ScrollPhase, TouchEvent,
    };
    pub use crate::permissions::{PermissionKind, PermissionStatus};
    pub use crate::platform::Platform;
    pub use crate::window::{Cursor, Window, WindowConfig, WindowId};
}
