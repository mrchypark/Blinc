//! Example
//!
//! A Blinc UI application with desktop, Android, iOS, and HarmonyOS support.
//! Demonstrates counter interactions and keyframe canvas animations.

mod sensor_inspector;

use blinc_app::prelude::*;
use blinc_app::windowed::WindowedContext;
use blinc_core::reactive::State;
use blinc_core::{Brush, DrawContext, Gradient};
use std::f32::consts::PI;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ExampleTheme {
    pub(crate) page_bg: Color,
    pub(crate) section_bg: Color,
    pub(crate) card_bg: Color,
    pub(crate) panel_bg: Color,
    pub(crate) text_primary: Color,
    pub(crate) text_secondary: Color,
    pub(crate) text_muted: Color,
    pub(crate) button_idle: Color,
    pub(crate) button_hovered: Color,
    pub(crate) button_pressed: Color,
    pub(crate) button_disabled: Color,
    pub(crate) accent: Color,
    pub(crate) sensor_idle: Color,
    pub(crate) sensor_hovered: Color,
    pub(crate) sensor_pressed: Color,
    pub(crate) sensor_active_idle: Color,
    pub(crate) sensor_active_hovered: Color,
    pub(crate) sensor_active_pressed: Color,
}

impl ExampleTheme {
    pub(crate) fn for_appearance(is_dark: bool) -> Self {
        if is_dark {
            Self {
                page_bg: Color::rgba(0.08, 0.08, 0.12, 1.0),
                section_bg: Color::rgba(0.12, 0.12, 0.17, 1.0),
                card_bg: Color::rgba(0.18, 0.18, 0.23, 1.0),
                panel_bg: Color::rgba(0.22, 0.22, 0.28, 1.0),
                text_primary: Color::WHITE,
                text_secondary: Color::rgba(0.84, 0.88, 0.96, 1.0),
                text_muted: Color::rgba(0.58, 0.60, 0.68, 1.0),
                button_idle: Color::rgba(0.30, 0.30, 0.40, 1.0),
                button_hovered: Color::rgba(0.38, 0.38, 0.50, 1.0),
                button_pressed: Color::rgba(0.22, 0.22, 0.30, 1.0),
                button_disabled: Color::rgba(0.20, 0.20, 0.20, 0.5),
                accent: Color::rgba(0.40, 0.80, 1.0, 1.0),
                sensor_idle: Color::rgba(0.30, 0.30, 0.40, 1.0),
                sensor_hovered: Color::rgba(0.40, 0.40, 0.50, 1.0),
                sensor_pressed: Color::rgba(0.20, 0.20, 0.30, 1.0),
                sensor_active_idle: Color::rgba(0.18, 0.38, 0.26, 1.0),
                sensor_active_hovered: Color::rgba(0.22, 0.46, 0.31, 1.0),
                sensor_active_pressed: Color::rgba(0.12, 0.28, 0.19, 1.0),
            }
        } else {
            Self {
                page_bg: Color::rgba(0.95, 0.97, 1.0, 1.0),
                section_bg: Color::rgba(1.0, 1.0, 1.0, 1.0),
                card_bg: Color::rgba(0.92, 0.95, 1.0, 1.0),
                panel_bg: Color::rgba(0.96, 0.97, 1.0, 1.0),
                text_primary: Color::rgba(0.12, 0.16, 0.25, 1.0),
                text_secondary: Color::rgba(0.24, 0.30, 0.40, 1.0),
                text_muted: Color::rgba(0.42, 0.48, 0.58, 1.0),
                button_idle: Color::rgba(0.78, 0.84, 0.94, 1.0),
                button_hovered: Color::rgba(0.70, 0.78, 0.90, 1.0),
                button_pressed: Color::rgba(0.62, 0.70, 0.84, 1.0),
                button_disabled: Color::rgba(0.80, 0.82, 0.86, 0.7),
                accent: Color::rgba(0.18, 0.50, 0.88, 1.0),
                sensor_idle: Color::rgba(0.78, 0.84, 0.94, 1.0),
                sensor_hovered: Color::rgba(0.70, 0.78, 0.90, 1.0),
                sensor_pressed: Color::rgba(0.62, 0.70, 0.84, 1.0),
                sensor_active_idle: Color::rgba(0.52, 0.78, 0.62, 1.0),
                sensor_active_hovered: Color::rgba(0.46, 0.72, 0.57, 1.0),
                sensor_active_pressed: Color::rgba(0.38, 0.64, 0.50, 1.0),
            }
        }
    }
}

pub(crate) fn content_padding_for_safe_area(_insets: (f32, f32, f32, f32)) -> (f32, f32) {
    (16.0, 20.0)
}

fn current_theme() -> ExampleTheme {
    ExampleTheme::for_appearance(current_is_dark_mode())
}

#[cfg(target_os = "ios")]
fn current_is_dark_mode() -> bool {
    blinc_platform_ios::is_dark_mode()
}

#[cfg(not(target_os = "ios"))]
fn current_is_dark_mode() -> bool {
    true
}

#[cfg(target_os = "ios")]
fn current_safe_area_insets() -> (f32, f32, f32, f32) {
    blinc_platform_ios::get_safe_area_insets()
}

#[cfg(not(target_os = "ios"))]
fn current_safe_area_insets() -> (f32, f32, f32, f32) {
    (0.0, 0.0, 0.0, 0.0)
}

/// Counter button with stateful hover/press states
fn counter_button(
    label: &str,
    count: State<i32>,
    delta: i32,
    theme: ExampleTheme,
) -> impl ElementBuilder {
    let label = label.to_string();

    let count = count.clone();
    stateful::<ButtonState>()
        .on_state(move |ctx| {
            let bg = match ctx.state() {
                ButtonState::Idle => theme.button_idle,
                ButtonState::Hovered => theme.button_hovered,
                ButtonState::Pressed => theme.button_pressed,
                ButtonState::Disabled => theme.button_disabled,
            };

            div()
                .w(80.0)
                .h(50.0)
                .rounded(8.0)
                .bg(bg)
                .items_center()
                .justify_center()
                .cursor(CursorStyle::Pointer)
                .child(text(&label).size(24.0).color(theme.text_primary))
        })
        .on_click(move |_| {
            count.set(count.get() + delta);
        })
}

/// Counter display that reacts to count changes
fn counter_display(count: State<i32>, theme: ExampleTheme) -> impl ElementBuilder {
    stateful::<NoState>()
        .deps([count.signal_id()])
        .on_state(move |_ctx| {
            div().child(
                text(format!("Count: {}", count.get()))
                    .size(48.0)
                    .color(theme.accent)
                    .align(TextAlign::Center),
            )
        })
}

/// Counter demo section
fn counter_section(ctx: &WindowedContext, theme: ExampleTheme) -> Div {
    let count = ctx.use_state_keyed("count", || 0i32);

    section_card("Counter Demo", theme)
        .child(counter_display(count.clone(), theme))
        .child(
            div()
                .flex_row()
                .gap(16.0)
                .child(counter_button("-", count.clone(), -1, theme))
                .child(counter_button("+", count.clone(), 1, theme)),
        )
}

/// Soft-keyboard test section.
///
/// Tap the text field to focus it. On Android the IME pops up via
/// `app.show_soft_input()` from `android.rs`; on iOS the
/// `BlincKeyboardHelper.shared.showKeyboard()` Swift bridge is
/// invoked from `ios.rs` via `dlsym`-resolved
/// `blinc_ios_show_keyboard`. Tap outside (or call
/// `clear_focus()`) to dismiss.
///
/// **iOS prerequisite for soft keyboard**: copy
/// `extensions/blinc_platform_ios/templates/BlincNativeBridge.swift`
/// into your iOS project and add it to the Xcode build target. The
/// runner falls back to a no-op when the symbol isn't present, so
/// the link still succeeds without it — text input still works at
/// the model level (cursor moves, characters insert via hardware
/// keyboard / Bluetooth keyboard) but the soft keyboard won't pop
/// up.
fn keyboard_section(ctx: &WindowedContext) -> Div {
    // Persistent text-input state, keyed so it survives rebuilds.
    // `text_input_state_with_placeholder` returns a
    // `SharedTextInputState` (Arc<Mutex<TextInputData>>) which is
    // cloned into the widget on every rebuild — the underlying
    // buffer + cursor position + focus flag persist across
    // rebuilds because the Arc points at the same allocation.
    let input_state = ctx.use_state_keyed("kbd_test_input", || {
        text_input_state_with_placeholder("Tap to focus and type…")
    });

    // Surrounding container is dark, so the input's idle and
    // focused background colors both need to be dark too —
    // otherwise the focus state defaults to the theme's
    // `InputBgFocus` (light, designed for light-mode pages) and
    // the white text becomes invisible on a white field.
    let input_bg = Color::rgba(0.20, 0.20, 0.27, 1.0);
    let input_focus_bg = Color::rgba(0.24, 0.24, 0.32, 1.0);

    section_card("Soft Keyboard Test")
        .child(
            text("Tap the field — the OS soft keyboard should pop up on mobile.")
                .size(13.0)
                .align(TextAlign::Center)
                .color(Color::rgba(0.7, 0.7, 0.78, 1.0)),
        )
        .child(
            div().w(300.0).child(
                text_input(&input_state.get())
                    .w_full()
                    .text_color(Color::WHITE)
                    .placeholder_color(Color::rgba(0.55, 0.55, 0.62, 1.0))
                    .idle_bg_color(input_bg)
                    .hover_bg_color(input_bg)
                    .focused_bg_color(input_focus_bg)
                    .border_color(Color::rgba(0.32, 0.32, 0.40, 1.0))
                    .focused_border_color(Color::rgba(0.45, 0.55, 0.95, 1.0)),
            ),
        )
}

/// Demo 1: Spinning loader using rotation keyframes
fn spinning_loader_demo(ctx: &WindowedContext, theme: ExampleTheme) -> Div {
    let timeline = ctx.use_animated_timeline();

    let entry_id = timeline.lock().unwrap().configure(|t| {
        let entry = t.add(0, 1000, 0.0, 360.0);
        t.set_loop(-1);
        t.start();
        entry
    });

    let render_timeline = Arc::clone(&timeline);

    demo_card("Spinning Loader", theme).child(
        canvas(move |ctx: &mut dyn DrawContext, bounds| {
            let timeline = render_timeline.lock().unwrap();
            let angle_deg = timeline.get(entry_id).unwrap_or(0.0);
            let angle_rad = angle_deg * PI / 180.0;

            let cx = bounds.width / 2.0;
            let cy = bounds.height / 2.0;
            let radius = 30.0;
            let thickness = 4.0;

            let arc_length = PI * 1.5;
            let segments = 32;

            for i in 0..segments {
                let t1 = i as f32 / segments as f32;
                let t2 = (i + 1) as f32 / segments as f32;

                let a1 = angle_rad + t1 * arc_length;
                let a2 = angle_rad + t2 * arc_length;

                let x1 = cx + radius * a1.cos();
                let y1 = cy + radius * a1.sin();
                let _x2 = cx + radius * a2.cos();
                let _y2 = cy + radius * a2.sin();

                let dx = _x2 - x1;
                let dy = _y2 - y1;
                let len = (dx * dx + dy * dy).sqrt();

                let alpha = 0.3 + 0.7 * t1;

                ctx.fill_rect(
                    Rect::new(
                        x1 - thickness / 2.0,
                        y1 - thickness / 2.0,
                        len + thickness,
                        thickness,
                    ),
                    blinc_core::CornerRadius::uniform(thickness / 2.0),
                    Brush::Solid(Color::rgba(0.4, 0.8, 1.0, alpha)),
                );
            }
        })
        .w(100.0)
        .h(100.0),
    )
}

/// Demo 2: Pulsing dots with staggered keyframes
fn pulsing_dots_demo(ctx: &WindowedContext, theme: ExampleTheme) -> Div {
    let timelines: Vec<SharedAnimatedTimeline> = (0..3)
        .map(|i| ctx.use_animated_timeline_for(format!("pulsing_dot_{}", i)))
        .collect();

    let entry_ids: Vec<_> = timelines
        .iter()
        .enumerate()
        .map(|(i, timeline)| {
            timeline
                .lock()
                .unwrap()
                .configure(|t: &mut AnimatedTimeline| {
                    let offset = i as i32 * 200;
                    let scale_entry = t.add(offset, 600, 0.5, 1.0);
                    let opacity_entry = t.add(offset, 600, 0.3, 1.0);
                    t.set_loop(-1);
                    t.start();
                    (scale_entry, opacity_entry)
                })
        })
        .collect();

    let timelines_clone: Vec<_> = timelines.iter().map(Arc::clone).collect();

    demo_card("Pulsing Dots", theme).child(
        canvas(move |ctx: &mut dyn DrawContext, bounds| {
            let cx = bounds.width / 2.0;
            let cy = bounds.height / 2.0;
            let dot_radius = 8.0;
            let spacing = 25.0;

            for (i, (timeline, (scale_entry, opacity_entry))) in
                timelines_clone.iter().zip(entry_ids.iter()).enumerate()
            {
                let tl = timeline.lock().unwrap();
                let scale = tl.get(*scale_entry).unwrap_or(1.0);
                let opacity = tl.get(*opacity_entry).unwrap_or(1.0);

                let x = cx + (i as f32 - 1.0) * spacing;
                let r = dot_radius * scale;

                ctx.fill_rect(
                    Rect::new(x - r, cy - r, r * 2.0, r * 2.0),
                    blinc_core::CornerRadius::uniform(r),
                    Brush::Solid(Color::rgba(0.4, 1.0, 0.8, opacity)),
                );
            }
        })
        .w(100.0)
        .h(100.0),
    )
}

/// Demo 3: Progress bar with eased fill animation
fn progress_bar_demo(ctx: &WindowedContext, theme: ExampleTheme) -> Div {
    let timeline = ctx.use_animated_timeline();

    let entry_id = timeline.lock().unwrap().configure(|t| {
        let entry = t.add(0, 2000, 0.0, 1.0);
        entry
    });

    let render_timeline = Arc::clone(&timeline);
    let click_timeline = Arc::clone(&timeline);
    let ready_timeline = Arc::clone(&timeline);

    ctx.query("progress-bar-demo").on_ready(move |_| {
        ready_timeline.lock().unwrap().start();
    });

    demo_card("Progress Bar", theme)
        .id("progress-bar-demo")
        .child(
            canvas(move |ctx: &mut dyn DrawContext, bounds| {
                let timeline = render_timeline.lock().unwrap();
                let progress_val = timeline.get(entry_id).unwrap_or(0.0);

                let bar_width = bounds.width - 20.0;
                let bar_height = 12.0;
                let bar_x = 10.0;
                let bar_y = (bounds.height - bar_height) / 2.0;

                // Background
                ctx.fill_rect(
                    Rect::new(bar_x, bar_y, bar_width, bar_height),
                    blinc_core::CornerRadius::uniform(6.0),
                    Brush::Solid(theme.panel_bg),
                );

                // Filled portion
                let fill_width = bar_width * progress_val;
                if fill_width > 0.0 {
                    ctx.fill_rect(
                        Rect::new(bar_x, bar_y, fill_width, bar_height),
                        blinc_core::CornerRadius::uniform(6.0),
                        Brush::Gradient(Gradient::linear(
                            Point::new(bar_x, bar_y),
                            Point::new(bar_x + fill_width, bar_y),
                            Color::rgba(0.4, 0.8, 1.0, 1.0),
                            Color::rgba(0.6, 0.4, 1.0, 1.0),
                        )),
                    );
                }
            })
            .w(150.0)
            .h(60.0),
        )
        .child(
            text("Tap to restart")
                .size(12.0)
                .color(theme.text_muted),
        )
        .on_click(move |_| {
            click_timeline.lock().unwrap().restart();
        })
}

/// Demo 4: Bouncing ball with squash and stretch
fn bouncing_ball_demo(ctx: &WindowedContext, theme: ExampleTheme) -> Div {
    let timeline = ctx.use_animated_timeline();

    let entry_id = timeline.lock().unwrap().configure(|t| {
        let y_entry = t.add(0, 800, 0.0, 1.0);
        t.set_loop(-1);
        t.start();
        y_entry
    });

    let render_timeline = Arc::clone(&timeline);

    demo_card("Bouncing Ball", theme).child(
        canvas(move |ctx: &mut dyn DrawContext, bounds| {
            let timeline = render_timeline.lock().unwrap();
            let t = timeline.get(entry_id).unwrap_or(0.0);

            let bounce_height = 50.0;
            let ground_y = bounds.height - 25.0;

            // Simple parabolic bounce
            let y = if t < 0.5 {
                let fall_t = t * 2.0;
                ground_y - bounce_height * (1.0 - fall_t * fall_t)
            } else {
                let rise_t = (t - 0.5) * 2.0;
                ground_y - bounce_height * (1.0 - (1.0 - rise_t) * (1.0 - rise_t))
            };

            // Squash/stretch based on velocity
            let (scale_x, scale_y) = if t < 0.45 || t > 0.55 {
                (0.9, 1.1)
            } else {
                (1.2, 0.8)
            };

            let cx = bounds.width / 2.0;
            let radius = 15.0;

            // Draw shadow
            let shadow_scale = 1.0 - (ground_y - y) / bounce_height * 0.5;
            let shadow_width = radius * 2.0 * shadow_scale;
            let shadow_height = radius * 0.3 * 2.0 * shadow_scale;
            ctx.fill_rect(
                Rect::new(
                    cx - shadow_width / 2.0,
                    ground_y + 2.0,
                    shadow_width,
                    shadow_height,
                ),
                blinc_core::CornerRadius::uniform(shadow_height / 2.0),
                Brush::Solid(Color::rgba(0.0, 0.0, 0.0, 0.3 * shadow_scale)),
            );

            // Draw ball with squash/stretch
            let ball_width = radius * 2.0 * scale_x;
            let ball_height = radius * 2.0 * scale_y;
            ctx.fill_rect(
                Rect::new(
                    cx - ball_width / 2.0,
                    y - ball_height / 2.0,
                    ball_width,
                    ball_height,
                ),
                blinc_core::CornerRadius::uniform(ball_height.min(ball_width) / 2.0),
                Brush::Gradient(Gradient::linear(
                    Point::new(cx - ball_width / 2.0, y - ball_height / 2.0),
                    Point::new(cx + ball_width / 2.0, y + ball_height / 2.0),
                    Color::rgba(1.0, 0.5, 0.3, 1.0),
                    Color::rgba(0.9, 0.3, 0.2, 1.0),
                )),
            );
        })
        .w(100.0)
        .h(120.0),
    )
}

/// Animation demos section
fn animation_section(ctx: &WindowedContext, theme: ExampleTheme) -> Div {
    section_card("Keyframe Animations", theme)
        .child(
            text("Canvas elements with multi-property keyframe animations")
                .size(16.0)
                .color(theme.text_muted)
                .align(TextAlign::Center),
        )
        .child(
            div()
                .flex_row()
                .gap(10.0)
                .flex_wrap()
                .justify_center()
                .child(spinning_loader_demo(ctx, theme))
                .child(pulsing_dots_demo(ctx, theme)),
        )
        .child(
            div()
                .flex_row()
                .gap(10.0)
                .flex_wrap()
                .justify_center()
                .child(progress_bar_demo(ctx, theme))
                .child(bouncing_ball_demo(ctx, theme)),
        )
}

/// Helper to create a section card
fn section_card(title: &str, theme: ExampleTheme) -> Div {
    div()
        .w_full()
        .flex_col()
        .gap(6.0)
        .py(5.0)
        .px(8.0)
        .bg(theme.section_bg)
        .rounded(16.0)
        .items_center()
        .child(
            div().items_center().child(
                text(title)
                    .size(24.0)
                    .align(TextAlign::Center)
                    .weight(FontWeight::Bold)
                    .color(theme.text_primary)
                    .no_wrap(),
            ),
        )
}

/// Helper to create a demo card
fn demo_card(title: &str, theme: ExampleTheme) -> Div {
    div()
        .w(170.0)
        .flex_col()
        .gap(5.0)
        .py(8.0)
        .px(4.0)
        .bg(theme.card_bg)
        .rounded(12.0)
        .items_center()
        .child(
            text(title)
                .size(14.0)
                .weight(FontWeight::SemiBold)
                .color(theme.text_primary),
        )
}

/// Main application UI with scroll container
fn app_ui(ctx: &mut WindowedContext) -> impl ElementBuilder {
    let theme = current_theme();
    let insets = current_safe_area_insets();
    let (top_padding, bottom_padding) = content_padding_for_safe_area(insets);

    div()
        .id("example.root")
        .w(ctx.width)
        .h(ctx.height)
        .bg(theme.page_bg)
        .child(
            scroll().id("example.scroll").w(ctx.width).h(ctx.height).child(
                div()
                    .id("example.content")
                    .w_full()
                    .flex_col()
                    .items_center()
                    .gap(4.0)
                    .px(8.0)
                    .pt(top_padding)
                    .pb(bottom_padding)
                    // Header
                    .child(
                        text("Blinc Mobile Example")
                            .id("example.header.title")
                            .align(TextAlign::Center)
                            .size(28.0)
                            .weight(FontWeight::Bold)
                            .color(theme.text_primary),
                    )
                    .child(
                        text("Scroll down for more demos")
                            .size(14.0)
                            .color(theme.text_muted),
                    )
                    // Counter section
                    .child(counter_section(ctx))
                    // Soft-keyboard test section
                    .child(keyboard_section(ctx))
                    // Animation section
                    .child(animation_section(ctx, theme)),
            ),
        )
}

// =============================================================================
// Desktop Entry Point
// =============================================================================

#[cfg(not(any(target_os = "android", target_os = "ios", target_env = "ohos")))]
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "Blinc Mobile Example".to_string(),
        width: 400,
        height: 700,
        ..Default::default()
    };

    blinc_app::windowed::WindowedApp::run(config, |ctx| app_ui(ctx))
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
            .with_tag("example"),
    );

    log::info!("Starting example on Android");

    // Wire up the JNI bridge to the Kotlin BlincNativeBridge object.
    // MainActivity.onCreate has already called BlincNativeBridge.registerDefaults
    // on the JVM side, so this just attaches the platform adapter on the Rust side.
    if let Err(e) = blinc_platform_android::init_android_native_bridge(&app) {
        log::warn!("Failed to init Android native bridge: {}", e);
    }

    blinc_app::AndroidApp::run(app, |ctx| app_ui(ctx)).expect("Failed to run Android app");
}

#[cfg(target_os = "android")]
fn main() {}

// =============================================================================
// iOS Entry Point
// =============================================================================

#[cfg(target_os = "ios")]
fn main() {}

/// iOS initialization function - called from Swift during app launch
///
/// This registers the Rust UI builder so that each frame can build the UI.
/// Must be called before blinc_create_context.
#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn ios_app_init() {
    use std::io::Write;
    // Filter out the chatty wgpu/naga shader-compilation crates so the
    // Xcode console only shows blinc + app traces. Without this filter
    // every shader compile dumps thousands of lines of `naga::front`
    // type-resolution debug output which drowns out touch-event traces.
    let filter = tracing_subscriber::EnvFilter::new(
        "info,\
         blinc_layout=debug,\
         blinc_app=debug,\
         blinc_animation=debug,\
         blinc_layout::widgets::text_input=debug,\
         blinc_layout::widgets::text_edit=debug,\
         wgpu=warn,\
         wgpu_core=warn,\
         wgpu_hal=warn,\
         naga=warn",
    );
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();

    eprintln!("[Blinc] ios_app_init called - registering UI builder");

    blinc_app::ios::register_rust_ui_builder(|ctx| app_ui(ctx));

    eprintln!("[Blinc] UI builder registered");
}

// =============================================================================
// HarmonyOS Entry Point
// =============================================================================

#[cfg(target_env = "ohos")]
fn main() {
    // HarmonyOS uses N-API callbacks from XComponent
    // The actual initialization happens via napi_register_module
    // This main() is a placeholder for the cdylib entry
}

/// N-API module export for HarmonyOS
/// Called when the native module is loaded by ArkTS
#[cfg(target_env = "ohos")]
#[no_mangle]
pub extern "C" fn napi_register_module() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    tracing::info!("Blinc HarmonyOS module registered");

    // TODO: Register N-API functions for XComponent callbacks
    // blinc_platform_harmony::napi_bridge::register_module()
}

#[cfg(test)]
mod tests {
    use blinc_app::RenderTree;
    use blinc_app::windowed::WindowedContext;
    use super::{content_padding_for_safe_area, ExampleTheme};

    #[test]
    fn light_and_dark_theme_palettes_are_distinct() {
        let dark = ExampleTheme::for_appearance(true);
        let light = ExampleTheme::for_appearance(false);

        assert_ne!(dark.page_bg, light.page_bg);
        assert_ne!(dark.card_bg, light.card_bg);
        assert_ne!(dark.text_primary, light.text_primary);
    }

    #[test]
    fn content_padding_uses_tighter_top_spacing_than_before() {
        let (top, bottom) = content_padding_for_safe_area((24.0, 0.0, 34.0, 0.0));

        assert_eq!(top, 16.0, "expected default top spacing to stay stable");
        assert_eq!(bottom, 20.0, "expected default bottom spacing to stay stable");
    }

    #[test]
    fn content_padding_keeps_legacy_top_spacing_without_safe_area() {
        let (top, bottom) = content_padding_for_safe_area((0.0, 0.0, 0.0, 0.0));

        assert_eq!(top, 16.0, "expected zero-inset layouts to keep the old top spacing");
        assert_eq!(bottom, 20.0, "expected zero-inset layouts to keep the old bottom spacing");
    }

    #[test]
    fn header_title_stays_near_top_in_headless_layout() {
        let mut ctx = WindowedContext::new_headless(393.0, 852.0);
        let ui = super::app_ui(&mut ctx);
        let mut tree = RenderTree::from_element_with_registry(&ui, ctx.element_registry().clone());
        tree.compute_layout(ctx.width, ctx.height);

        let title = tree
            .query_by_id("example.header.title")
            .expect("header title should exist");
        let bounds = tree
            .get_bounds(title)
            .expect("header title should have layout bounds");

        assert!(
            bounds.y < 120.0,
            "header title should start near the top, got y={}",
            bounds.y
        );
    }

    #[test]
    fn scroll_content_fills_at_least_the_viewport_height() {
        let mut ctx = WindowedContext::new_headless(393.0, 852.0);
        let ui = super::app_ui(&mut ctx);
        let mut tree = RenderTree::from_element_with_registry(&ui, ctx.element_registry().clone());
        tree.compute_layout(ctx.width, ctx.height);

        let content = tree
            .query_by_id("example.content")
            .expect("content root should exist");
        let bounds = tree
            .get_bounds(content)
            .expect("content root should have layout bounds");

        assert!(
            bounds.height >= ctx.height,
            "expected content to fill the viewport height, got height={} viewport={}",
            bounds.height,
            ctx.height
        );
    }
}
