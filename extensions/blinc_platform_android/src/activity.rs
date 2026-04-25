//! Android Activity integration
//!
//! Provides the main entry point for Android applications and handles
//! the Android activity lifecycle events.

#[cfg(target_os = "android")]
use android_activity::{AndroidApp, MainEvent, PollEvent};

#[cfg(target_os = "android")]
use ndk::native_window::NativeWindow;

#[cfg(target_os = "android")]
use tracing::{debug, info, warn};

#[derive(Debug, Default)]
struct AndroidRenderState {
    has_window: bool,
    focused: bool,
    running: bool,
    redraw_requested: bool,
    last_surface_size: Option<(u32, u32)>,
}

impl AndroidRenderState {
    fn new() -> Self {
        Self {
            running: true,
            ..Self::default()
        }
    }

    fn on_window_attached(&mut self, width: u32, height: u32) {
        self.has_window = true;
        self.last_surface_size = Some((width, height));
        self.redraw_requested = true;
    }

    fn on_window_resized(&mut self, width: u32, height: u32) {
        self.last_surface_size = Some((width, height));
        self.redraw_requested = true;
    }

    fn on_window_detached(&mut self) {
        self.has_window = false;
        self.redraw_requested = false;
    }

    fn on_focus_changed(&mut self, focused: bool) {
        self.focused = focused;
        if focused && self.has_window {
            self.redraw_requested = true;
        }
    }

    fn on_resume(&mut self) {
        if self.focused && self.has_window {
            self.redraw_requested = true;
        }
    }

    fn on_pause(&mut self) {
        self.focused = false;
    }

    fn on_destroy(&mut self) {
        self.running = false;
        self.redraw_requested = false;
    }

    fn mark_low_memory(&mut self) {
        // Low-memory is advisory. Keep any already-queued redraw request intact.
    }

    fn should_render(&self) -> bool {
        self.running && self.has_window && self.focused && self.redraw_requested
    }

    fn mark_rendered(&mut self) {
        self.redraw_requested = false;
    }
}

/// Android application state
pub struct BlincAndroidApp {
    #[cfg(target_os = "android")]
    window: Option<NativeWindow>,
    render_state: AndroidRenderState,
}

impl BlincAndroidApp {
    /// Create a new Blinc Android application
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "android")]
            window: None,
            render_state: AndroidRenderState::new(),
        }
    }

    /// Render a frame
    pub fn render_frame(&mut self) {
        // TODO: Render using blinc_gpu
        // 1. Process reactive updates
        // 2. Update animations
        // 3. Layout widgets
        // 4. Paint to GPU
        self.render_state.mark_rendered();
    }

    /// Check if app is running
    pub fn is_running(&self) -> bool {
        self.render_state.running
    }

    /// Check if we should render
    pub fn should_render(&self) -> bool {
        self.render_state.should_render()
    }
}

#[cfg(target_os = "android")]
impl BlincAndroidApp {
    /// Handle Android events
    pub fn handle_event(&mut self, app: &AndroidApp, event: PollEvent) {
        if let PollEvent::Main(main_event) = event {
            self.handle_main_event(app, main_event);
        }
    }

    /// Handle main lifecycle events
    fn handle_main_event(&mut self, app: &AndroidApp, event: MainEvent) {
        match event {
            MainEvent::InitWindow { .. } => {
                info!("Native window initialized");
                // Get the native window
                if let Some(window) = app.native_window() {
                    let width = window.width();
                    let height = window.height();
                    info!("Window size: {}x{}", width, height);
                    self.render_state
                        .on_window_attached(width as u32, height as u32);
                    self.window = Some(window);
                    self.init_graphics();
                }
            }

            MainEvent::TerminateWindow { .. } => {
                info!("Native window terminated");
                self.window = None;
                self.render_state.on_window_detached();
            }

            MainEvent::WindowResized { .. } => {
                if let Some(ref window) = self.window {
                    let width = window.width();
                    let height = window.height();
                    info!("Window resized: {}x{}", width, height);
                    self.render_state
                        .on_window_resized(width as u32, height as u32);
                }
            }

            MainEvent::GainedFocus => {
                info!("App gained focus");
                self.render_state.on_focus_changed(true);
            }

            MainEvent::LostFocus => {
                info!("App lost focus");
                self.render_state.on_focus_changed(false);
            }

            MainEvent::Pause => {
                info!("App paused");
                self.render_state.on_pause();
            }

            MainEvent::Resume { .. } => {
                info!("App resumed");
                self.render_state.on_resume();
            }

            MainEvent::Start => {
                info!("App started");
            }

            MainEvent::Stop => {
                info!("App stopped");
            }

            MainEvent::Destroy => {
                info!("App destroyed");
                self.render_state.on_destroy();
            }

            MainEvent::SaveState { .. } => {
                debug!("Saving app state");
                // TODO: Save reactive state
            }

            MainEvent::ConfigChanged { .. } => {
                debug!("Configuration changed");
            }

            MainEvent::LowMemory => {
                warn!("Low memory warning");
                self.render_state.mark_low_memory();
            }

            MainEvent::ContentRectChanged { .. } => {
                debug!("Content rect changed");
            }

            _ => {}
        }
    }

    /// Initialize graphics (GPU renderer)
    fn init_graphics(&mut self) {
        if let Some(ref _window) = self.window {
            // TODO: Initialize wgpu with the native window
            // This will use blinc_gpu with Vulkan backend
            info!("Graphics initialization placeholder");
        }
    }
}

#[cfg(target_os = "android")]
impl Default for BlincAndroidApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Android main entry point
///
/// This is called by the android-activity crate when the app starts.
/// This is only enabled when the "default-activity" feature is enabled.
/// Applications should typically provide their own android_main and use
/// blinc_app::AndroidApp::run() instead.
#[cfg(all(target_os = "android", feature = "default-activity"))]
#[no_mangle]
pub fn android_main(app: AndroidApp) {
    // Initialize Android logging
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("Blinc"),
    );

    info!("android_main called");

    let mut blinc_app = BlincAndroidApp::new();

    while blinc_app.is_running() {
        // Poll for events with 16ms timeout (roughly 60fps)
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            blinc_app.handle_event(&app, event);
        });

        // Render if we have a window and are focused
        if blinc_app.should_render() {
            blinc_app.render_frame();
        }
    }

    info!("Blinc Android app shutting down");
}

/// Placeholder for non-Android builds (allows cross-compilation checks)
#[cfg(not(target_os = "android"))]
pub fn android_main() {
    // This is never called - just allows the code to compile on non-Android
}

#[cfg(test)]
mod tests {
    use super::{android_main, AndroidRenderState, BlincAndroidApp};

    // Tests run on host, not on Android
    #[test]
    fn test_placeholder() {
        // Android-specific code can't be tested on host
    }
}
