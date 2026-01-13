//! {{project_name}}
//!
//! A Blinc UI application with desktop, Android, and iOS support.

use blinc_app::prelude::*;

/// Main application UI
fn app_ui(ctx: &mut WindowedContext) -> impl ElementBuilder {
    let count = ctx.use_state(|| 0i32);

    div()
        .w_full()
        .h_full()
        .bg([0.1, 0.1, 0.15, 1.0])
        .flex_col()
        .items_center()
        .justify_center()
        .gap(20.0)
        .child(
            text("{{project_name}}")
                .size(32.0)
                .color([1.0, 1.0, 1.0, 1.0]),
        )
        .child(
            text(format!("Count: {}", count.get()))
                .size(48.0)
                .color([0.4, 0.8, 1.0, 1.0]),
        )
        .child(
            div()
                .flex_row()
                .gap(16.0)
                .child(
                    button("-")
                        .on_click({
                            let count = count.clone();
                            move |_| count.set(count.get() - 1)
                        })
                        .padding(16.0, 32.0),
                )
                .child(
                    button("+")
                        .on_click({
                            let count = count.clone();
                            move |_| count.set(count.get() + 1)
                        })
                        .padding(16.0, 32.0),
                ),
        )
}

// =============================================================================
// Desktop Entry Point
// =============================================================================

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn main() {
    WindowedApp::run(app_ui).expect("Failed to run desktop app");
}

// =============================================================================
// Android Entry Point
// =============================================================================

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use android_logger::Config;
    use log::LevelFilter;

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("{{project_name}}"),
    );

    log::info!("Starting {{project_name}} on Android");

    blinc_app::AndroidApp::run(app, app_ui).expect("Failed to run Android app");
}

// Dummy main for Android (required by Rust but not used)
#[cfg(target_os = "android")]
fn main() {}

// =============================================================================
// iOS Entry Point
// =============================================================================

#[cfg(target_os = "ios")]
fn main() {
    // iOS entry is handled via C FFI from Swift
    // See platforms/ios/BlincApp/AppDelegate.swift
}

/// iOS: Create render context (called from Swift)
#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_blinc_app(
    width: u32,
    height: u32,
    scale_factor: f64,
) -> *mut std::ffi::c_void {
    match blinc_app::IOSApp::create(width, height, scale_factor, app_ui) {
        Ok(app) => Box::into_raw(Box::new(app)) as *mut std::ffi::c_void,
        Err(e) => {
            eprintln!("Failed to create Blinc app: {:?}", e);
            std::ptr::null_mut()
        }
    }
}

// =============================================================================
// UI Components
// =============================================================================

/// Simple button component
fn button(label: &str) -> impl ElementBuilder {
    let label = label.to_string();

    div()
        .bg([0.3, 0.3, 0.4, 1.0])
        .rounded(8.0)
        .cursor_pointer()
        .child(text(label).size(24.0).color([1.0, 1.0, 1.0, 1.0]))
}

trait ButtonExt: Sized {
    fn padding(self, vertical: f32, horizontal: f32) -> Self;
}

impl<T: ElementBuilder> ButtonExt for T {
    fn padding(self, vertical: f32, horizontal: f32) -> Self {
        self.px(horizontal).py(vertical)
    }
}
