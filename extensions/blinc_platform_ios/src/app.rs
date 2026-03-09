//! iOS application integration
//!
//! Provides the iOS platform implementation and entry point.

use crate::event_loop::IOSEventLoop;
use crate::window::IOSWindow;
use blinc_platform::{Platform, PlatformError};

#[cfg(target_os = "ios")]
use objc2_foundation::MainThreadMarker;
#[cfg(target_os = "ios")]
use objc2_ui_kit::{UIApplication, UIScreen, UIUserInterfaceStyle, UIWindow};

#[cfg(target_os = "ios")]
use tracing::info;

fn resolve_dark_mode(value: Option<bool>) -> bool {
    value.unwrap_or(false)
}

fn resolve_safe_area_insets(value: Option<(f32, f32, f32, f32)>) -> (f32, f32, f32, f32) {
    value.unwrap_or((0.0, 0.0, 0.0, 0.0))
}

#[cfg(target_os = "ios")]
#[allow(deprecated)]
fn active_window() -> Option<objc2::rc::Retained<UIWindow>> {
    let mtm = MainThreadMarker::new()?;
    let application = UIApplication::sharedApplication(mtm);

    application
        .keyWindow()
        .or_else(|| application.windows().firstObject())
}

#[cfg(target_os = "ios")]
pub(crate) fn current_window() -> Option<IOSWindow> {
    let window = active_window()?;
    let scale_factor = window.screen().scale() as f64;
    let bounds = window.bounds();
    let width = (bounds.size.width * scale_factor).round() as u32;
    let height = (bounds.size.height * scale_factor).round() as u32;
    Some(IOSWindow::new(window, scale_factor, width, height))
}

#[cfg(target_os = "ios")]
fn query_dark_mode() -> Option<bool> {
    let window = active_window()?;
    let scene = window.windowScene()?;
    let traits = scene.traitCollection();
    let style = unsafe { traits.userInterfaceStyle() };
    Some(style == UIUserInterfaceStyle::Dark)
}

#[cfg(target_os = "ios")]
fn query_safe_area_insets() -> Option<(f32, f32, f32, f32)> {
    let window = active_window()?;

    if let Some(controller) = window.rootViewController() {
        controller.loadViewIfNeeded();
        if let Some(view) = controller.view() {
            let insets = view.safeAreaInsets();
            return Some((
                insets.top as f32,
                insets.left as f32,
                insets.bottom as f32,
                insets.right as f32,
            ));
        }
    }

    let insets = window.safeAreaInsets();
    Some((
        insets.top as f32,
        insets.left as f32,
        insets.bottom as f32,
        insets.right as f32,
    ))
}

/// iOS platform implementation
pub struct IOSPlatform {
    /// Display scale factor
    #[cfg(target_os = "ios")]
    scale_factor: f64,
    #[cfg(not(target_os = "ios"))]
    _private: (),
}

#[cfg(target_os = "ios")]
impl IOSPlatform {
    /// Get the main screen's scale factor
    #[allow(deprecated)]
    fn get_screen_scale() -> f64 {
        // On iOS, UIScreen::mainScreen() requires MainThreadMarker
        // We assume this is called from the main thread
        let mtm = MainThreadMarker::new().expect("Must be called from main thread");
        let screen = UIScreen::mainScreen(mtm);
        screen.scale() as f64
    }
}

impl Platform for IOSPlatform {
    type Window = IOSWindow;
    type EventLoop = IOSEventLoop;

    #[cfg(target_os = "ios")]
    fn new() -> Result<Self, PlatformError> {
        let scale_factor = Self::get_screen_scale();
        info!(
            "iOS platform initialized with scale factor: {}",
            scale_factor
        );
        Ok(Self { scale_factor })
    }

    #[cfg(not(target_os = "ios"))]
    fn new() -> Result<Self, PlatformError> {
        Err(PlatformError::Unsupported(
            "iOS platform only available on iOS".to_string(),
        ))
    }

    fn create_event_loop(&self) -> Result<Self::EventLoop, PlatformError> {
        Ok(IOSEventLoop::new())
    }

    fn name(&self) -> &'static str {
        "ios"
    }

    #[cfg(target_os = "ios")]
    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    #[cfg(not(target_os = "ios"))]
    fn scale_factor(&self) -> f64 {
        1.0
    }
}

/// iOS main entry point
///
/// This function is called by the iOS app delegate when the application launches.
/// It initializes the Blinc runtime and sets up the Metal rendering context.
///
/// Note: On iOS, the actual event loop is managed by UIKit's RunLoop.
/// This function sets up the necessary infrastructure for Blinc to integrate
/// with the iOS lifecycle.
#[cfg(target_os = "ios")]
pub fn ios_main() {
    info!("iOS main entry point called");

    // Note: The actual application lifecycle is managed by UIApplicationDelegate
    // and UIKit. This function serves as the Rust-side initialization point.
    //
    // A typical iOS app using Blinc would:
    // 1. Create a UIWindow with a UIViewController
    // 2. Set up a CAMetalLayer on the view
    // 3. Create a CADisplayLink for frame callbacks
    // 4. Route touch events to Blinc's input system
    //
    // See IOSApp::run() in blinc_app for the full integration.
}

/// Placeholder for non-iOS builds
#[cfg(not(target_os = "ios"))]
pub fn ios_main() {
    // Placeholder - iOS main is only called on iOS
}

/// Get the display scale factor for the main screen
#[cfg(target_os = "ios")]
pub fn get_display_scale() -> f64 {
    IOSPlatform::get_screen_scale()
}

/// Placeholder for non-iOS builds
#[cfg(not(target_os = "ios"))]
pub fn get_display_scale() -> f64 {
    1.0
}

/// Check if the system is in dark mode
#[cfg(target_os = "ios")]
pub fn is_dark_mode() -> bool {
    resolve_dark_mode(query_dark_mode())
}

/// Placeholder for non-iOS builds
#[cfg(not(target_os = "ios"))]
pub fn is_dark_mode() -> bool {
    resolve_dark_mode(None)
}

/// Get the safe area insets for the main screen
///
/// Returns (top, left, bottom, right) insets in logical points.
#[cfg(target_os = "ios")]
pub fn get_safe_area_insets() -> (f32, f32, f32, f32) {
    resolve_safe_area_insets(query_safe_area_insets())
}

/// Placeholder for non-iOS builds
#[cfg(not(target_os = "ios"))]
pub fn get_safe_area_insets() -> (f32, f32, f32, f32) {
    resolve_safe_area_insets(None)
}

/// iOS system font paths
///
/// These are the common font locations on iOS. Note that different iOS versions
/// and simulator vs device may have fonts at different paths. The fonts in the
/// Core directory are the most reliable across different iOS versions.
pub fn system_font_paths() -> &'static [&'static str] {
    &[
        // iOS system fonts - Core directory (most reliable)
        "/System/Library/Fonts/Core/SFUI.ttf", // SF UI (system font)
        "/System/Library/Fonts/Core/SFUIMono.ttf", // SF Mono
        "/System/Library/Fonts/Core/SFUIItalic.ttf", // SF Italic
        "/System/Library/Fonts/Core/Helvetica.ttc", // Helvetica
        "/System/Library/Fonts/Core/HelveticaNeue.ttc", // Helvetica Neue
        "/System/Library/Fonts/Core/Avenir.ttc", // Avenir
        "/System/Library/Fonts/Core/AvenirNext.ttc", // Avenir Next
        "/System/Library/Fonts/Core/Courier.ttc", // Courier
        "/System/Library/Fonts/Core/CourierNew.ttf", // Courier New
        // CoreUI fonts
        "/System/Library/Fonts/CoreUI/Menlo.ttc", // Menlo (monospace)
        "/System/Library/Fonts/CoreUI/SFUIRounded.ttf", // SF Rounded
        // CoreAddition fonts
        "/System/Library/Fonts/CoreAddition/Georgia.ttf",
        "/System/Library/Fonts/CoreAddition/Arial.ttf",
        "/System/Library/Fonts/CoreAddition/ArialBold.ttf",
        "/System/Library/Fonts/CoreAddition/Verdana.ttf",
        "/System/Library/Fonts/CoreAddition/TimesNewRomanPS.ttf",
    ]
}

#[cfg(test)]
mod tests {
    use super::{resolve_dark_mode, resolve_safe_area_insets};

    #[test]
    fn dark_mode_defaults_to_light_when_unavailable() {
        assert!(!resolve_dark_mode(None));
    }

    #[test]
    fn dark_mode_preserves_detected_value() {
        assert!(resolve_dark_mode(Some(true)));
        assert!(!resolve_dark_mode(Some(false)));
    }

    #[test]
    fn safe_area_defaults_to_zeroes_when_unavailable() {
        assert_eq!(resolve_safe_area_insets(None), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn safe_area_preserves_detected_insets() {
        assert_eq!(
            resolve_safe_area_insets(Some((10.0, 4.0, 34.0, 4.0))),
            (10.0, 4.0, 34.0, 4.0)
        );
    }
}
