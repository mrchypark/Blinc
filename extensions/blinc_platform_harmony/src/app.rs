//! HarmonyOS platform implementation
//!
//! Implements the Platform trait for HarmonyOS using XComponent.

use blinc_platform::{Platform, PlatformError};

use crate::event_loop::HarmonyEventLoop;
use crate::window::HarmonyWindow;

/// HarmonyOS platform using XComponent
pub struct HarmonyPlatform {
    /// Display scale factor
    scale_factor: f64,
}

impl HarmonyPlatform {
    /// Get system font paths for HarmonyOS
    pub fn system_font_paths() -> &'static [&'static str] {
        &[
            "/system/fonts/HarmonyOS_Sans_SC_Regular.ttf",
            "/system/fonts/Roboto-Regular.ttf",
            "/system/fonts/NotoSansCJK-Regular.ttc",
        ]
    }

    /// Create a new HarmonyOS platform with the given scale factor
    pub fn new_with_scale(scale_factor: f64) -> Self {
        Self { scale_factor }
    }
}

impl Platform for HarmonyPlatform {
    type Window = HarmonyWindow;
    type EventLoop = HarmonyEventLoop;

    fn new() -> Result<Self, PlatformError> {
        // TODO: Query display info from HarmonyOS display module
        Ok(Self { scale_factor: 1.0 })
    }

    fn name(&self) -> &'static str {
        "harmony"
    }

    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    fn create_event_loop(&self) -> Result<Self::EventLoop, PlatformError> {
        Ok(HarmonyEventLoop::new())
    }
}
