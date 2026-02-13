//! Windowed application runner
//!
//! Provides a unified API for running windowed Blinc applications across
//! desktop and Android platforms.
//!
//! # Example
//!
//! ```ignore
//! use blinc_app::prelude::*;
//! use blinc_app::windowed::WindowedApp;
//!
//! fn main() -> Result<()> {
//!     WindowedApp::run(WindowConfig::default(), |ctx| {
//!         // Build your UI using reactive signals
//!         let count = ctx.use_signal(0);
//!         let doubled = ctx.use_derived(move |cx| cx.get(count).unwrap_or(0) * 2);
//!
//!         div().w_full().h_full()
//!             .flex_center()
//!             .child(text(&format!("Count: {}", ctx.get(count).unwrap_or(0))).size(48.0))
//!     })
//! }
//! ```

use std::hash::Hash;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use blinc_animation::{
    AnimatedTimeline, AnimatedValue, AnimationContext, AnimationScheduler, SchedulerHandle,
    SharedAnimatedTimeline, SharedAnimatedValue, SpringConfig,
};
use blinc_core::context_state::{BlincContextState, HookState, SharedHookState, StateKey};
use blinc_core::reactive::{Derived, ReactiveGraph, Signal, SignalId, State, StatefulDepsCallback};
use blinc_layout::overlay_state::{get_overlay_manager, OverlayContext};
use blinc_layout::prelude::*;
use blinc_layout::widgets::overlay::{overlay_manager, OverlayManager, OverlayManagerExt};
use blinc_platform::{
    ControlFlow, Event, EventLoop, InputEvent, Key, KeyState, LifecycleEvent, MouseEvent, Platform,
    TouchEvent, Window, WindowConfig, WindowEvent,
};
#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
use blinc_platform::{
    PlatformError, WebView, WebViewBounds, WebViewConfig, WebViewEvent, WebViewHost,
    WebViewNavigationBlockReason, WebViewNavigationDecision, WebViewNavigationPolicy,
};

use crate::app::BlincApp;
use crate::error::{BlincError, Result};

// -----------------------------------------------------------------------------
// Optional windowed e2e (desktop)
// -----------------------------------------------------------------------------
//
// We can't rely on OS-level screenshots in CI (or on machines without Screen
// Recording permission). For deterministic macOS e2e we optionally read back the
// rendered swapchain frame (requires SurfaceConfiguration.usage COPY_SRC) and
// validate pixels directly.
//
// Enabled via env vars:
// - BLINC_E2E_CAPTURE_PATH=/tmp/out.png   (write PNG; can be a directory too)
// - BLINC_E2E_EXPECT=blueish|warm         (assert minimal pixels in main panel)
// - BLINC_E2E_TRIGGER_PATH=/tmp/trigger   (optional: only capture when this file exists)
// - BLINC_E2E_MAX_CAPTURES=2              (optional: number of captures before exiting)
// - BLINC_E2E_EXIT=0|false                (optional: keep running after captures)
// - BLINC_E2E_SCRIPT=gallery_sidebar_click_after_scroll (optional: internal input simulation)
// - BLINC_E2E_SCRIPT_EXIT=0|false          (optional: keep running after script)
//
// Note: this is intentionally minimal and only runs once per process.

fn e2e_is_enabled() -> bool {
    std::env::var_os("BLINC_E2E_CAPTURE_PATH").is_some()
        || std::env::var_os("BLINC_E2E_EXPECT").is_some()
}

fn e2e_exit_after() -> bool {
    if let Ok(v) = std::env::var("BLINC_E2E_EXIT") {
        let v = v.trim().to_ascii_lowercase();
        return !(v.is_empty() || v == "0" || v == "false" || v == "no");
    }
    e2e_is_enabled()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum E2eScript {
    GallerySidebarClickAfterScroll,
}

fn e2e_script() -> Option<E2eScript> {
    let v = std::env::var("BLINC_E2E_SCRIPT").ok()?;
    match v.trim().to_ascii_lowercase().as_str() {
        "gallery_sidebar_click_after_scroll" | "gallery-sidebar-click-after-scroll" => {
            Some(E2eScript::GallerySidebarClickAfterScroll)
        }
        _ => None,
    }
}

fn e2e_script_exit_after(script_enabled: bool) -> bool {
    if !script_enabled {
        return false;
    }
    if let Ok(v) = std::env::var("BLINC_E2E_SCRIPT_EXIT") {
        let v = v.trim().to_ascii_lowercase();
        return !(v.is_empty() || v == "0" || v == "false" || v == "no");
    }
    true
}

fn read_keyed_state<T: Clone + Send + 'static>(
    hooks: &SharedHookState,
    reactive: &SharedReactiveGraph,
    key: &str,
) -> Option<T> {
    let state_key = StateKey::from_string::<T>(key);
    let raw_id = hooks.lock().ok()?.get(&state_key)?;
    let signal_id = SignalId::from_raw(raw_id);
    let signal: Signal<T> = Signal::from_id(signal_id);
    reactive.lock().ok()?.get(signal)
}

fn e2e_find_hit_point_with_id_prefix(
    tree: &RenderTree,
    router: &EventRouter,
    window_w: f32,
    window_h: f32,
    x_max: f32,
    id_prefix: &str,
) -> Option<(f32, f32, String)> {
    let x0 = 6.0;
    let y0 = 6.0;
    let x1 = x_max.min(window_w - 6.0);
    let y1 = (window_h - 6.0).max(y0);
    let step = 8.0;

    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            if let Some(hit) = router.hit_test(tree, x, y) {
                if let Some(id) = tree.element_registry().get_id(hit.node) {
                    if id.starts_with(id_prefix) {
                        return Some((x, y, id.to_string()));
                    }
                }
            }
            x += step;
        }
        y += step;
    }
    None
}

fn e2e_find_hit_point_with_exact_id(
    tree: &RenderTree,
    router: &EventRouter,
    window_w: f32,
    window_h: f32,
    x_max: f32,
    exact_id: &str,
) -> Option<(f32, f32)> {
    let x0 = 6.0;
    let y0 = 6.0;
    let x1 = x_max.min(window_w - 6.0);
    let y1 = (window_h - 6.0).max(y0);
    let step = 8.0;

    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            if let Some(hit) = router.hit_test(tree, x, y) {
                if let Some(id) = tree.element_registry().get_id(hit.node) {
                    if id == exact_id {
                        return Some((x, y));
                    }
                }
            }
            x += step;
        }
        y += step;
    }
    None
}

fn e2e_max_captures() -> usize {
    std::env::var("BLINC_E2E_MAX_CAPTURES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v >= 1)
        .unwrap_or(1)
}

fn e2e_trigger_path() -> Option<std::path::PathBuf> {
    std::env::var("BLINC_E2E_TRIGGER_PATH")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
}

fn e2e_capture_on_start(trigger: Option<&std::path::PathBuf>) -> bool {
    if let Ok(v) = std::env::var("BLINC_E2E_CAPTURE_ON_START") {
        let v = v.trim().to_ascii_lowercase();
        return !(v.is_empty() || v == "0" || v == "false" || v == "no");
    }
    // If an explicit trigger path is configured, default to trigger-only.
    trigger.is_none()
}

fn e2e_output_path(base: &std::path::Path, capture_index: usize) -> std::path::PathBuf {
    // capture_index is 1-based.
    if base.is_dir() {
        return base.join(format!("capture-{capture_index}.png"));
    }

    if capture_index <= 1 {
        return base.to_path_buf();
    }

    let file_name = base
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("capture.png");
    let (stem, ext) = match file_name.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{e}")),
        None => (file_name.to_string(), String::new()),
    };
    let new_name = format!("{stem}-{capture_index}{ext}");
    base.with_file_name(new_name)
}

fn padded_bytes_per_row(width: u32) -> u32 {
    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    unpadded.div_ceil(align) * align
}

fn bgra_or_rgba_to_rgba(
    format: wgpu::TextureFormat,
    bytes: &[u8],
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    let bytes_per_row = padded_bytes_per_row(width) as usize;
    let expected = bytes_per_row.saturating_mul(height as usize);
    if bytes.len() < expected {
        return None;
    }

    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    for y in 0..height as usize {
        let row_start = y * bytes_per_row;
        let row_end = row_start + (width as usize) * 4;
        let row = &bytes[row_start..row_end];
        let dst = &mut out[y * (width as usize) * 4..(y + 1) * (width as usize) * 4];

        match format {
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                for x in 0..width as usize {
                    let i = x * 4;
                    // BGRA -> RGBA
                    dst[i] = row[i + 2];
                    dst[i + 1] = row[i + 1];
                    dst[i + 2] = row[i];
                    dst[i + 3] = row[i + 3];
                }
            }
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {
                dst.copy_from_slice(row);
            }
            _ => return None,
        }
    }
    Some(out)
}

fn e2e_save_png_minimal_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    path: &std::path::Path,
) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &b in data {
            crc ^= b as u32;
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (0xEDB8_8320u32 & mask);
            }
        }
        !crc
    }

    fn write_chunk(file: &mut File, typ: &[u8; 4], data: &[u8]) -> std::io::Result<()> {
        file.write_all(&(data.len() as u32).to_be_bytes())?;
        file.write_all(typ)?;
        file.write_all(data)?;
        let mut crc_data = Vec::with_capacity(typ.len() + data.len());
        crc_data.extend_from_slice(typ);
        crc_data.extend_from_slice(data);
        file.write_all(&crc32(&crc_data).to_be_bytes())?;
        Ok(())
    }

    fn adler32_rgba(rgba: &[u8], width: u32, height: u32) -> u32 {
        let mut a: u32 = 1;
        let mut b: u32 = 0;
        let mod_adler: u32 = 65521;

        let row_len = width as usize * 4;
        for y in 0..height as usize {
            // Each scanline begins with a single filter byte (0 = None).
            a %= mod_adler;
            b = (b + a) % mod_adler;
            let row = &rgba[y * row_len..(y + 1) * row_len];
            for &byte in row {
                a = (a + byte as u32) % mod_adler;
                b = (b + a) % mod_adler;
            }
        }
        (b << 16) | a
    }

    let mut file = File::create(path)?;

    // PNG signature
    file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])?;

    // IHDR
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type RGBA
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace
    write_chunk(&mut file, b"IHDR", &ihdr)?;

    // IDAT: uncompressed deflate blocks
    let mut idat = Vec::new();
    let scanline_len = width as usize * 4 + 1;
    idat.push(0x78);
    idat.push(0x01);

    for y in 0..height as usize {
        let is_last = y + 1 == height as usize;
        let bfinal = if is_last { 1u8 } else { 0u8 };

        let row_start = y * (width as usize) * 4;
        let row_end = row_start + (width as usize) * 4;

        let mut scanline = Vec::with_capacity(scanline_len);
        scanline.push(0); // filter: none
        scanline.extend_from_slice(&rgba[row_start..row_end]);

        idat.push(bfinal); // BFINAL + BTYPE=00
        let len = scanline.len() as u16;
        idat.extend_from_slice(&len.to_le_bytes());
        idat.extend_from_slice(&(!len).to_le_bytes());
        idat.extend_from_slice(&scanline);
    }

    let adler = adler32_rgba(rgba, width, height);
    idat.extend_from_slice(&adler.to_be_bytes());

    write_chunk(&mut file, b"IDAT", &idat)?;
    write_chunk(&mut file, b"IEND", &[])?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum E2eExpect {
    Blueish,
    Warm,
}

fn e2e_expect() -> Option<E2eExpect> {
    let v = std::env::var("BLINC_E2E_EXPECT").ok()?;
    match v.trim().to_ascii_lowercase().as_str() {
        "blue" | "blueish" | "line" => Some(E2eExpect::Blueish),
        "warm" | "heat" | "heatmap" => Some(E2eExpect::Warm),
        _ => None,
    }
}

fn e2e_threshold(expect: E2eExpect, w: u32, h: u32, total_samples: usize) -> usize {
    // Use a threshold proportional to window size so it scales across Retina/non-Retina.
    // These are intentionally conservative: we mostly want to catch “nothing rendered”.
    let area = (w as u64).saturating_mul(h as u64).max(1);
    match expect {
        E2eExpect::Blueish => {
            // Line: the stroke is thin, but should still appear across a wide span.
            // Require at least ~0.03% of sampled points to be “colored”.
            (total_samples / 3000)
                .max(10)
                .min((area / 50_000) as usize + 20)
        }
        E2eExpect::Warm => {
            // Heatmap fills a lot of area; expect more colored pixels.
            (total_samples / 800).max(30)
        }
    }
}

fn e2e_count_pixels(rgba: &[u8], w: u32, h: u32) -> (usize, usize, usize) {
    // Returns (blueish, warm, total_samples).
    if w == 0 || h == 0 {
        return (0, 0, 0);
    }

    let iw = w as i32;
    let ih = h as i32;

    // Sample the main content area, avoiding sidebar/tabs where selection
    // colors could cause false positives.
    //
    // Important: `w`/`h` are physical pixels. On Retina macOS they are typically 2x the
    // logical window size, so we prefer a logical-size hint when available.
    let narrow = std::env::var("BLINC_WINDOW_SIZE")
        .ok()
        .and_then(|v| v.trim().split_once('x').map(|(a, _b)| a.trim().to_string()))
        .and_then(|w| w.parse::<i32>().ok())
        .map(|logical_w| logical_w < 900)
        .unwrap_or(iw < 900);
    // In narrow layouts we have the tabs row on top; the plot starts lower.
    let x0 = if narrow {
        (iw as f32 * 0.06)
    } else {
        (iw as f32 * 0.33)
    } as i32;
    let x1 = (iw as f32 * 0.97) as i32;
    let y0 = if narrow {
        (ih as f32 * 0.42) as i32
    } else {
        (ih as f32 * 0.24) as i32
    };
    // In narrow layouts, the chart can sit very close to the bottom edge (e.g. heatmap),
    // so we sample a little lower to avoid false negatives.
    let y1 = if narrow {
        (ih as f32 * 0.98) as i32
    } else {
        (ih as f32 * 0.92) as i32
    };

    let step_x = ((x1 - x0) / 120).max(4);
    let step_y = ((y1 - y0) / 80).max(4);

    let mut blue = 0usize;
    let mut warm = 0usize;
    let mut total = 0usize;

    let row_bytes = w as usize * 4;
    for y in (y0.max(0)..y1.min(ih)).step_by(step_y as usize) {
        let row = (y as usize) * row_bytes;

        // For multi-line charts the “signal” is thin and multi-colored. A coarse grid
        // can miss it; scan a subset of columns per row instead of sampling just a few points.
        for x in (x0.max(0)..x1.min(iw)).step_by(2) {
            total += 1;
            let idx = row + (x as usize) * 4;
            if idx + 4 > rgba.len() {
                continue;
            }
            let r = rgba[idx] as f32 / 255.0;
            let g = rgba[idx + 1] as f32 / 255.0;
            let b = rgba[idx + 2] as f32 / 255.0;
            let a = rgba[idx + 3] as f32 / 255.0;

            if a >= 0.8 {
                // “Blueish” is really “non-background, colored stroke” in the main plot.
                // Use a broader heuristic:
                // - not too dark
                // - noticeable chroma (channel spread) OR strong blue dominance
                let mx = r.max(g).max(b);
                let mn = r.min(g).min(b);
                let spread = mx - mn;
                let colored = (mx > 0.22 && spread > 0.08) || (b > 0.35 && (b - r.max(g)) > 0.10);
                if colored {
                    blue += 1;
                }
                // Warm heatmap pixels skew red/yellow: high (r,g), lower b.
                // Keep this tolerant; we mainly want to catch “heatmap disappeared”.
                if (r > 0.50) && (g > 0.18) && (b < 0.60) && (r - b > 0.18) && (g - b > 0.10) {
                    warm += 1;
                }
            }
        }
    }

    (blue, warm, total)
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
struct DesktopWebViewLifecycle {
    host: Option<blinc_platform_desktop::DesktopWebViewHost>,
    webview: Option<blinc_platform_desktop::DesktopWebView>,
    create_attempted: bool,
    navigation_policy: WebViewNavigationPolicy,
    navigation_url: Option<String>,
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
const WEBVIEW_URL_ENV: &str = "BLINC_WEBVIEW_URL";

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
const WEBVIEW_ALLOW_ORIGINS_ENV: &str = "BLINC_WEBVIEW_ALLOW_ORIGINS";

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
impl Default for DesktopWebViewLifecycle {
    fn default() -> Self {
        Self {
            host: None,
            webview: None,
            create_attempted: false,
            navigation_policy: webview_navigation_policy_from_env(),
            navigation_url: webview_navigation_url_from_env(),
        }
    }
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
fn webview_navigation_url_from_env() -> Option<String> {
    std::env::var(WEBVIEW_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
fn webview_navigation_policy_from_env() -> WebViewNavigationPolicy {
    let Some(raw_origins) = std::env::var(WEBVIEW_ALLOW_ORIGINS_ENV).ok() else {
        return WebViewNavigationPolicy::default();
    };

    raw_origins
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .fold(WebViewNavigationPolicy::default(), |policy, origin| {
            policy.allow_origin(origin)
        })
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
fn apply_navigation_policy(
    config: WebViewConfig,
    requested_url: Option<&str>,
) -> (WebViewConfig, Option<WebViewNavigationDecision>) {
    let Some(url) = requested_url.map(str::trim).filter(|url| !url.is_empty()) else {
        return (config, None);
    };

    let decision = config.navigation_policy.evaluate(url);
    let config = match &decision {
        WebViewNavigationDecision::Allowed { .. } => config.initial_url(url),
        WebViewNavigationDecision::Blocked { .. } => config,
    };

    (config, Some(decision))
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
fn navigation_block_reason_code(reason: &WebViewNavigationBlockReason) -> &'static str {
    match reason {
        WebViewNavigationBlockReason::MalformedUrl => "malformed-url",
        WebViewNavigationBlockReason::OriginNotAllowed => "origin-not-allowed",
    }
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
fn navigation_decision_event_name(decision: &WebViewNavigationDecision) -> &'static str {
    match decision {
        WebViewNavigationDecision::Allowed { .. } => "navigation allowed",
        WebViewNavigationDecision::Blocked { .. } => "navigation blocked",
    }
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
fn log_webview_navigation_decision(decision: &WebViewNavigationDecision) {
    let event_name = navigation_decision_event_name(decision);
    match decision {
        WebViewNavigationDecision::Allowed { url, origin } => {
            tracing::info!(
                target: "blinc_webview_policy",
                event = event_name,
                url = %url,
                origin = %origin,
                "{event_name}"
            );
        }
        WebViewNavigationDecision::Blocked {
            url,
            origin,
            reason,
        } => {
            tracing::warn!(
                target: "blinc_webview_policy",
                event = event_name,
                url = %url,
                origin = %origin.as_deref().unwrap_or(""),
                reason = navigation_block_reason_code(reason),
                "{event_name}"
            );
        }
    }
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
impl DesktopWebViewLifecycle {
    fn ensure_created(&mut self, window: &blinc_platform_desktop::DesktopWindow) {
        if self.host.is_none() {
            self.host = Some(window.webview_host());
        }
        if self.webview.is_some() || self.create_attempted {
            return;
        }

        self.create_attempted = true;
        let (width, height) = window.logical_size();
        let bounds = WebViewBounds::new(0.0, 0.0, width, height);
        let config = WebViewConfig::new()
            .bounds(bounds)
            .navigation_policy(self.navigation_policy.clone());
        let (config, decision) = apply_navigation_policy(config, self.navigation_url.as_deref());
        if let Some(ref decision) = decision {
            log_webview_navigation_decision(decision);
        }

        let Some(host) = self.host.as_ref() else {
            return;
        };

        match host.create_webview(config) {
            Ok(mut webview) => {
                if let Err(error) = webview.set_event_handler(Some(Box::new(|event| match event {
                    WebViewEvent::Message(message) => {
                        tracing::debug!("Desktop webview message received: {}", message)
                    }
                    WebViewEvent::Error(error) => {
                        tracing::warn!("Desktop webview reported error: {:?}", error)
                    }
                    _ => {}
                }))) {
                    log_webview_error("register event handler", &error);
                }
                self.webview = Some(webview);
                tracing::info!("Desktop webview created");
            }
            Err(error) => {
                log_webview_error("create", &error);
            }
        }
    }

    fn sync_bounds(&self, window: &blinc_platform_desktop::DesktopWindow) {
        if let Some(host) = self.host.as_ref() {
            if let Err(error) = host.sync_bounds_with_window() {
                log_webview_error("sync host bounds", &error);
            }
        }

        if let Some(webview) = self.webview.as_ref() {
            let (width, height) = window.logical_size();
            let bounds = WebViewBounds::new(0.0, 0.0, width, height);
            if let Err(error) = webview.set_bounds(bounds) {
                log_webview_error("set bounds", &error);
            }
        }
    }

    fn on_focus_changed(&self, focused: bool) {
        if let Some(webview) = self.webview.as_ref() {
            let message = if focused {
                r#"{"type":"window-focus","focused":true}"#
            } else {
                r#"{"type":"window-focus","focused":false}"#
            };
            if let Err(error) = webview.post_message(message) {
                log_webview_error("post focus message", &error);
            }
        }
    }

    fn dispose(&mut self) {
        if let Some(mut webview) = self.webview.take() {
            if let Err(error) = webview.destroy() {
                log_webview_error("destroy", &error);
            }
        }

        if let Some(host) = self.host.take() {
            if let Err(error) = host.cleanup() {
                log_webview_error("cleanup", &error);
            }
        }

        self.create_attempted = false;
    }
}

#[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
fn log_webview_error(action: &str, error: &PlatformError) {
    match error {
        PlatformError::Unavailable(_) => {
            tracing::info!(
                "Desktop webview {} unavailable; continuing without webview: {}",
                action,
                error
            );
        }
        _ => {
            tracing::warn!(
                "Desktop webview {} failed; continuing without webview: {}",
                action,
                error
            );
        }
    }
}

/// Shared animation scheduler for the application (thread-safe)
pub type SharedAnimationScheduler = Arc<Mutex<AnimationScheduler>>;

// SharedAnimatedValue and SharedAnimatedTimeline are re-exported from blinc_animation

#[cfg(all(feature = "windowed", not(target_os = "android")))]
use blinc_platform_desktop::DesktopPlatform;

/// Shared dirty flag type for element refs
pub type RefDirtyFlag = Arc<AtomicBool>;

/// Shared reactive graph for the application (thread-safe)
pub type SharedReactiveGraph = Arc<Mutex<ReactiveGraph>>;

/// Shared element registry for query API (thread-safe)
pub type SharedElementRegistry = Arc<blinc_layout::selector::ElementRegistry>;

/// Callback type for on_ready handlers
pub type ReadyCallback = Box<dyn FnOnce() + Send + Sync>;

/// Shared storage for ready callbacks
pub type SharedReadyCallbacks = Arc<Mutex<Vec<ReadyCallback>>>;

/// Context passed to the UI builder function
pub struct WindowedContext {
    /// Current window width in logical pixels (for UI layout)
    ///
    /// This is the width you should use when building UI. It automatically
    /// accounts for DPI scaling, so elements sized to `ctx.width` will
    /// fill the window regardless of display scale factor.
    pub width: f32,
    /// Current window height in logical pixels (for UI layout)
    pub height: f32,
    /// Current scale factor (physical / logical)
    pub scale_factor: f64,
    /// Physical window width (for internal use)
    pub(crate) physical_width: f32,
    /// Physical window height (for internal use)
    pub(crate) physical_height: f32,
    /// Whether the window is focused
    pub focused: bool,
    /// Number of completed UI rebuilds (0 = first build in progress)
    ///
    /// Use `is_ready()` to check if the UI has been built at least once.
    /// This is useful for triggering animations after motion bindings are registered.
    pub rebuild_count: u32,
    /// Event router for input event handling
    pub event_router: EventRouter,
    /// Animation scheduler for spring/keyframe animations
    pub animations: SharedAnimationScheduler,
    /// Shared dirty flag for element refs - when set, triggers UI rebuild
    ref_dirty_flag: RefDirtyFlag,
    /// Reactive graph for signal-based state management
    reactive: SharedReactiveGraph,
    /// Hook state for call-order based signal persistence
    hooks: SharedHookState,
    /// Overlay manager for modals, dialogs, toasts, etc.
    overlay_manager: OverlayManager,
    /// Whether overlays were visible last frame (for triggering rebuilds)
    had_visible_overlays: bool,
    /// Element registry for query API (shared with RenderTree)
    element_registry: SharedElementRegistry,
    /// Callbacks to run after UI is ready (motion bindings registered)
    ready_callbacks: SharedReadyCallbacks,
    /// CSS stylesheet for automatic style application (hover, animations, base styles)
    /// Multiple stylesheets cascade — later rules override earlier ones.
    pub stylesheet: Option<Arc<blinc_layout::css_parser::Stylesheet>>,
}

impl WindowedContext {
    #[allow(clippy::too_many_arguments)]
    fn from_window<W: Window>(
        window: &W,
        event_router: EventRouter,
        animations: SharedAnimationScheduler,
        ref_dirty_flag: RefDirtyFlag,
        reactive: SharedReactiveGraph,
        hooks: SharedHookState,
        overlay_mgr: OverlayManager,
        element_registry: SharedElementRegistry,
        ready_callbacks: SharedReadyCallbacks,
    ) -> Self {
        // Get physical size (actual surface pixels) and scale factor
        let (physical_width, physical_height) = window.size();
        let scale_factor = window.scale_factor();

        // Compute logical size (what users work with in their UI code)
        // This ensures elements sized with ctx.width/height fill the window
        // regardless of DPI, and font sizes appear consistent across displays
        let logical_width = physical_width as f32 / scale_factor as f32;
        let logical_height = physical_height as f32 / scale_factor as f32;

        Self {
            width: logical_width,
            height: logical_height,
            scale_factor,
            physical_width: physical_width as f32,
            physical_height: physical_height as f32,
            focused: window.is_focused(),
            rebuild_count: 0,
            event_router,
            animations,
            ref_dirty_flag,
            reactive,
            hooks,
            overlay_manager: overlay_mgr,
            had_visible_overlays: false,
            element_registry,
            ready_callbacks,
            stylesheet: None,
        }
    }

    /// Create a WindowedContext for Android
    ///
    /// This is used by the Android runner since it doesn't have a Window trait implementation.
    #[cfg(all(feature = "android", target_os = "android"))]
    pub(crate) fn new_android(
        logical_width: f32,
        logical_height: f32,
        scale_factor: f64,
        physical_width: f32,
        physical_height: f32,
        focused: bool,
        animations: SharedAnimationScheduler,
        ref_dirty_flag: RefDirtyFlag,
        reactive: SharedReactiveGraph,
        hooks: SharedHookState,
        overlay_mgr: OverlayManager,
        element_registry: SharedElementRegistry,
        ready_callbacks: SharedReadyCallbacks,
    ) -> Self {
        Self {
            width: logical_width,
            height: logical_height,
            scale_factor,
            physical_width,
            physical_height,
            focused,
            rebuild_count: 0,
            event_router: EventRouter::new(),
            animations,
            ref_dirty_flag,
            reactive,
            hooks,
            overlay_manager: overlay_mgr,
            had_visible_overlays: false,
            element_registry,
            ready_callbacks,
            stylesheet: None,
        }
    }

    /// Create a WindowedContext for iOS
    ///
    /// This is used by the iOS runner since it doesn't have a Window trait implementation.
    #[cfg(all(feature = "ios", target_os = "ios"))]
    pub(crate) fn new_ios(
        logical_width: f32,
        logical_height: f32,
        scale_factor: f64,
        physical_width: f32,
        physical_height: f32,
        focused: bool,
        animations: SharedAnimationScheduler,
        ref_dirty_flag: RefDirtyFlag,
        reactive: SharedReactiveGraph,
        hooks: SharedHookState,
        overlay_mgr: OverlayManager,
        element_registry: SharedElementRegistry,
        ready_callbacks: SharedReadyCallbacks,
    ) -> Self {
        Self {
            width: logical_width,
            height: logical_height,
            scale_factor,
            physical_width,
            physical_height,
            focused,
            rebuild_count: 0,
            event_router: EventRouter::new(),
            animations,
            ref_dirty_flag,
            reactive,
            hooks,
            overlay_manager: overlay_mgr,
            had_visible_overlays: false,
            element_registry,
            ready_callbacks,
            stylesheet: None,
        }
    }

    /// Create a WindowedContext for Fuchsia
    ///
    /// This is used by the Fuchsia runner since it doesn't have a Window trait implementation.
    #[cfg(all(feature = "fuchsia", target_os = "fuchsia"))]
    pub(crate) fn new_fuchsia(
        logical_width: f32,
        logical_height: f32,
        scale_factor: f64,
        physical_width: f32,
        physical_height: f32,
        focused: bool,
        animations: SharedAnimationScheduler,
        ref_dirty_flag: RefDirtyFlag,
        reactive: SharedReactiveGraph,
        hooks: SharedHookState,
        overlay_mgr: OverlayManager,
        element_registry: SharedElementRegistry,
        ready_callbacks: SharedReadyCallbacks,
    ) -> Self {
        Self {
            width: logical_width,
            height: logical_height,
            scale_factor,
            physical_width,
            physical_height,
            focused,
            rebuild_count: 0,
            event_router: EventRouter::new(),
            animations,
            ref_dirty_flag,
            reactive,
            hooks,
            overlay_manager: overlay_mgr,
            had_visible_overlays: false,
            element_registry,
            ready_callbacks,
            stylesheet: None,
        }
    }

    /// Update context from window (preserving event router, dirty flag, and reactive graph)
    fn update_from_window<W: Window>(&mut self, window: &W) {
        let (physical_width, physical_height) = window.size();
        let scale_factor = window.scale_factor();

        self.physical_width = physical_width as f32;
        self.physical_height = physical_height as f32;
        self.width = physical_width as f32 / scale_factor as f32;
        self.height = physical_height as f32 / scale_factor as f32;
        self.scale_factor = scale_factor;
        self.focused = window.is_focused();
    }

    // =========================================================================
    // DPI-Related Helpers
    // =========================================================================

    /// Get the physical window width (for advanced use cases)
    ///
    /// Most users should use `ctx.width` which is in logical pixels.
    /// Physical dimensions are only needed when directly interfacing
    /// with GPU surfaces or platform-specific code.
    pub fn physical_width(&self) -> f32 {
        self.physical_width
    }

    /// Get the physical window height (for advanced use cases)
    pub fn physical_height(&self) -> f32 {
        self.physical_height
    }

    /// Check if the UI is ready (has completed at least one rebuild)
    ///
    /// This is useful for triggering animations after the first UI build,
    /// when motion bindings have been registered with the renderer.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn my_component(ctx: &WindowedContext) -> impl ElementBuilder {
    ///     let progress = ctx.use_animated_value_for("progress", 0.0, SpringConfig::gentle());
    ///
    ///     // Only trigger animation after UI is ready
    ///     let triggered = ctx.use_state_keyed("triggered", || false);
    ///     if ctx.is_ready() && !triggered.get() {
    ///         triggered.set(true);
    ///         progress.lock().unwrap().set_target(100.0);
    ///     }
    ///
    ///     // ... build UI ...
    /// }
    /// ```
    pub fn is_ready(&self) -> bool {
        self.rebuild_count > 0
    }

    /// Register a callback to run once after the UI is ready
    ///
    /// The callback will be executed after the first UI rebuild completes,
    /// when motion bindings have been registered with the renderer.
    /// This is the recommended way to trigger initial animations.
    ///
    /// Callbacks are executed once and then discarded. If `is_ready()` is
    /// already true, the callback will run on the next frame.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn my_component(ctx: &WindowedContext) -> impl ElementBuilder {
    ///     let progress = ctx.use_animated_value_for("progress", 0.0, SpringConfig::gentle());
    ///
    ///     // Register animation to trigger when UI is ready
    ///     let progress_clone = progress.clone();
    ///     ctx.on_ready(move || {
    ///         if let Ok(mut value) = progress_clone.lock() {
    ///             value.set_target(100.0);
    ///         }
    ///     });
    ///
    ///     // ... build UI ...
    /// }
    /// ```
    /// Register a callback to run once when the UI is ready (context-level).
    ///
    /// **Note:** For element-specific callbacks, prefer using the query API:
    /// ```ignore
    /// ctx.query_element("my-element").on_ready(|bounds| {
    ///     // Triggered once after element is laid out
    /// });
    /// ```
    /// The query-based approach uses stable string IDs that survive tree rebuilds.
    ///
    /// This context-level callback runs after the first rebuild completes.
    /// If called after the UI is already ready, executes immediately.
    pub fn on_ready<F>(&self, callback: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        // If already ready, execute immediately
        if self.rebuild_count > 0 {
            callback();
            return;
        }
        // Otherwise queue for execution after first rebuild
        if let Ok(mut callbacks) = self.ready_callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    // =========================================================================
    // Reactive Signal API
    // =========================================================================

    /// Create a persistent state value that survives across UI rebuilds (keyed)
    ///
    /// This creates component-level state identified by a unique string key.
    /// Returns a `State<T>` with direct `.get()` and `.set()` methods.
    ///
    /// For stateful UI elements with `StateTransitions`, prefer `use_state(initial)`
    /// which auto-keys by source location.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn my_button(ctx: &WindowedContext, id: &str) -> impl ElementBuilder {
    ///     // Each button gets its own hover state, keyed by id
    ///     let hovered = ctx.use_state_keyed(id, || false);
    ///
    ///     div()
    ///         .bg(if hovered.get() { Color::RED } else { Color::BLUE })
    ///         .on_hover_enter({
    ///             let hovered = hovered.clone();
    ///             move |_| hovered.set(true)
    ///         })
    ///         .on_hover_leave({
    ///             let hovered = hovered.clone();
    ///             move |_| hovered.set(false)
    ///         })
    /// }
    /// ```
    pub fn use_state_keyed<T, F>(&self, key: &str, init: F) -> State<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        use blinc_core::reactive::SignalId;

        let state_key = StateKey::from_string::<T>(key);
        // IMPORTANT: Do not execute `init()` while holding internal locks.
        // Otherwise, `init()` may call ctx.use_* / State::get() and deadlock.
        let existing_raw_id = { self.hooks.lock().unwrap().get(&state_key) };

        let signal = if let Some(raw_id) = existing_raw_id {
            let signal_id = SignalId::from_raw(raw_id);
            Signal::from_id(signal_id)
        } else {
            let initial = init();
            let signal = self.reactive.lock().unwrap().create_signal(initial);
            let raw_id = signal.id().to_raw();
            self.hooks.lock().unwrap().insert(state_key, raw_id);
            signal
        };

        // Create callback for stateful deps notification
        let callback: StatefulDepsCallback = Arc::new(|signal_ids| {
            blinc_layout::check_stateful_deps(signal_ids);
        });

        State::with_stateful_callback(
            signal,
            Arc::clone(&self.reactive),
            Arc::clone(&self.ref_dirty_flag),
            callback,
        )
    }

    /// Create a persistent signal that survives across UI rebuilds (keyed)
    ///
    /// Unlike `use_signal()` which creates a new signal each call, this method
    /// persists the signal using a unique string key. Use this for simple
    /// reactive values that need to survive rebuilds.
    ///
    /// For FSM-based state with `StateTransitions`, use `use_state_keyed()` instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let current_index = ctx.use_signal_keyed("current_index", || 0usize);
    ///
    /// // Read the value
    /// let index = ctx.get(current_index).unwrap_or(0);
    ///
    /// // Set the value (in an event handler)
    /// ctx.set(current_index, 1);
    /// ```
    pub fn use_signal_keyed<T, F>(&self, key: &str, init: F) -> Signal<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        use blinc_core::reactive::SignalId;

        let state_key = StateKey::from_string::<T>(key);
        // Same locking rule as `use_state_keyed`: run `init()` lock-free.
        let existing_raw_id = { self.hooks.lock().unwrap().get(&state_key) };

        if let Some(raw_id) = existing_raw_id {
            let signal_id = SignalId::from_raw(raw_id);
            Signal::from_id(signal_id)
        } else {
            let initial = init();
            let signal = self.reactive.lock().unwrap().create_signal(initial);
            let raw_id = signal.id().to_raw();
            self.hooks.lock().unwrap().insert(state_key, raw_id);
            signal
        }
    }

    /// Create a persistent ScrollRef for programmatic scroll control
    ///
    /// This creates a ScrollRef that survives across UI rebuilds. Use `.bind()`
    /// on a scroll widget to connect it, then call methods like `.scroll_to()`
    /// to programmatically control scrolling.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    ///     let scroll_ref = ctx.use_scroll_ref("my_scroll");
    ///
    ///     div()
    ///         .child(
    ///             scroll()
    ///                 .bind(&scroll_ref)
    ///                 .child(items.iter().map(|i| div().id(format!("item-{}", i.id))))
    ///         )
    ///         .child(
    ///             button("Scroll to item 5").on_click({
    ///                 let scroll_ref = scroll_ref.clone();
    ///                 move |_| scroll_ref.scroll_to("item-5")
    ///             })
    ///         )
    /// }
    /// ```
    pub fn use_scroll_ref(&self, key: &str) -> blinc_layout::selector::ScrollRef {
        use blinc_core::reactive::SignalId;
        use blinc_layout::selector::{ScrollRef, SharedScrollRefInner, TriggerCallback};

        // Create a unique key for the scroll ref's inner state
        let state_key =
            StateKey::from_string::<SharedScrollRefInner>(&format!("scroll_ref:{}", key));
        let mut hooks = self.hooks.lock().unwrap();

        // Check if we have an existing signal with this key
        let (signal_id, inner) = if let Some(raw_id) = hooks.get(&state_key) {
            // Reconstruct the signal ID and get the inner state from the reactive graph
            let signal_id = SignalId::from_raw(raw_id);
            let inner = self
                .reactive
                .lock()
                .unwrap()
                .get_untracked(Signal::<SharedScrollRefInner>::from_id(signal_id))
                .unwrap_or_else(ScrollRef::new_inner);
            (signal_id, inner)
        } else {
            // First time - create a new inner state and store it in the reactive graph
            let new_inner = ScrollRef::new_inner();
            let signal = self
                .reactive
                .lock()
                .unwrap()
                .create_signal(Arc::clone(&new_inner));
            let raw_id = signal.id().to_raw();
            hooks.insert(state_key, raw_id);
            (signal.id(), new_inner)
        };

        drop(hooks);

        // ScrollRef doesn't need to trigger rebuilds - scroll operations are processed
        // every frame by process_pending_scroll_refs()
        let noop_trigger: TriggerCallback = Arc::new(|| {});

        ScrollRef::with_inner(inner, signal_id, noop_trigger)
    }

    /// Create a new reactive signal with an initial value (low-level API)
    ///
    /// **Note**: Prefer `use_state` in most cases, as it automatically
    /// persists signals across rebuilds.
    ///
    /// This method always creates a new signal. Use this for advanced
    /// cases where you manage signal lifecycle manually.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = ctx.use_signal(0);
    ///
    /// // In an event handler:
    /// ctx.set(count, ctx.get(count).unwrap_or(0) + 1);
    /// ```
    pub fn use_signal<T: Send + 'static>(&self, initial: T) -> Signal<T> {
        self.reactive.lock().unwrap().create_signal(initial)
    }

    /// Get the current value of a signal
    ///
    /// This automatically tracks the signal as a dependency when called
    /// within a derived computation or effect.
    pub fn get<T: Clone + 'static>(&self, signal: Signal<T>) -> Option<T> {
        self.reactive.lock().unwrap().get(signal)
    }

    /// Set the value of a signal, triggering reactive updates
    ///
    /// This will automatically trigger a UI rebuild.
    pub fn set<T: Send + 'static>(&self, signal: Signal<T>, value: T) {
        self.reactive.lock().unwrap().set(signal, value);
        // Mark dirty to trigger rebuild
        self.ref_dirty_flag.store(true, Ordering::SeqCst);
    }

    /// Update a signal using a function
    ///
    /// This is useful for incrementing counters or modifying state based
    /// on the current value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ctx.update(count, |n| n + 1);
    /// ```
    pub fn update<T: Clone + Send + 'static, F: FnOnce(T) -> T>(&self, signal: Signal<T>, f: F) {
        self.reactive.lock().unwrap().update(signal, f);
        self.ref_dirty_flag.store(true, Ordering::SeqCst);
    }

    /// Create a derived (computed) value
    ///
    /// Derived values are lazily computed and cached. They automatically
    /// track their signal dependencies and recompute when those signals change.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = ctx.use_signal(5);
    /// let doubled = ctx.use_derived(move |cx| cx.get(count).unwrap_or(0) * 2);
    ///
    /// assert_eq!(ctx.get_derived(doubled), Some(10));
    /// ```
    pub fn use_derived<T, F>(&self, compute: F) -> Derived<T>
    where
        T: Clone + Send + 'static,
        F: Fn(&ReactiveGraph) -> T + Send + 'static,
    {
        self.reactive.lock().unwrap().create_derived(compute)
    }

    /// Get the value of a derived computation
    pub fn get_derived<T: Clone + 'static>(&self, derived: Derived<T>) -> Option<T> {
        self.reactive.lock().unwrap().get_derived(derived)
    }

    /// Create an effect that runs when its dependencies change
    ///
    /// Effects are useful for side effects like logging, network requests,
    /// or syncing state with external systems.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = ctx.use_signal(0);
    ///
    /// ctx.use_effect(move |cx| {
    ///     let value = cx.get(count).unwrap_or(0);
    ///     println!("Count changed to: {}", value);
    /// });
    /// ```
    pub fn use_effect<F>(&self, run: F) -> blinc_core::reactive::Effect
    where
        F: FnMut(&ReactiveGraph) + Send + 'static,
    {
        self.reactive.lock().unwrap().create_effect(run)
    }

    /// Batch multiple signal updates into a single reactive update
    ///
    /// This is useful when updating multiple signals at once to avoid
    /// redundant recomputations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ctx.batch(|g| {
    ///     g.set(x, 10);
    ///     g.set(y, 20);
    ///     g.set(z, 30);
    /// });
    /// // Only one UI rebuild triggered
    /// ```
    pub fn batch<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ReactiveGraph) -> R,
    {
        let result = self.reactive.lock().unwrap().batch(f);
        self.ref_dirty_flag.store(true, Ordering::SeqCst);
        result
    }

    /// Get the shared reactive graph for advanced usage
    ///
    /// This is useful when you need to pass the graph to closures or
    /// store it for later use.
    pub fn reactive(&self) -> SharedReactiveGraph {
        Arc::clone(&self.reactive)
    }

    /// Create a new DivRef that will trigger rebuilds when modified
    ///
    /// Use this to create refs that can be mutated in event handlers.
    /// When you call `.borrow_mut()` or `.with_mut()` on the returned ref,
    /// the UI will automatically rebuild when the mutation completes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let card_ref = ctx.create_ref::<Div>();
    ///
    /// div()
    ///     .child(
    ///         div()
    ///             .bind(&card_ref)
    ///             .on_hover_enter({
    ///                 let r = card_ref.clone();
    ///                 move |_| {
    ///                     // This automatically triggers a rebuild
    ///                     r.with_mut(|d| *d = d.swap().bg(Color::RED));
    ///                 }
    ///             })
    ///     )
    /// ```
    pub fn create_ref<T>(&self) -> ElementRef<T> {
        ElementRef::with_dirty_flag(Arc::clone(&self.ref_dirty_flag))
    }

    /// Create a new DivRef (convenience method)
    pub fn div_ref(&self) -> DivRef {
        self.create_ref::<Div>()
    }

    /// Get the shared dirty flag for manual state management
    ///
    /// Use this when you want to create your own state types that trigger
    /// UI rebuilds when modified. When you modify state, set this flag to true.
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct MyState {
    ///     value: i32,
    ///     dirty_flag: RefDirtyFlag,
    /// }
    ///
    /// impl MyState {
    ///     fn set_value(&mut self, v: i32) {
    ///         self.value = v;
    ///         self.dirty_flag.store(true, Ordering::SeqCst);
    ///     }
    /// }
    /// ```
    pub fn dirty_flag(&self) -> RefDirtyFlag {
        Arc::clone(&self.ref_dirty_flag)
    }

    /// Get a handle to the animation scheduler for creating animated values
    ///
    /// Components use this handle to create `AnimatedValue`s that automatically
    /// register with the scheduler. The scheduler ticks all animations each frame
    /// and triggers UI rebuilds while animations are active.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_animation::{AnimatedValue, SpringConfig};
    ///
    /// let opacity = AnimatedValue::new(ctx.animations(), 1.0, SpringConfig::stiff());
    /// opacity.set_target(0.5); // Auto-registers and animates
    /// let current = opacity.get(); // Get interpolated value
    /// ```
    pub fn animation_handle(&self) -> SchedulerHandle {
        self.animations.lock().unwrap().handle()
    }

    /// Get the overlay manager for showing modals, dialogs, toasts, etc.
    ///
    /// The overlay manager provides a fluent API for creating overlays that
    /// render in a separate pass after the main UI tree, ensuring they always
    /// appear on top.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_layout::prelude::*;
    ///
    /// fn my_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    ///     let overlay_mgr = ctx.overlay_manager();
    ///
    ///     div()
    ///         .child(
    ///             button("Show Modal").on_click({
    ///                 let mgr = overlay_mgr.clone();
    ///                 move |_| {
    ///                     mgr.modal()
    ///                         .content(|| {
    ///                             div().p(20.0).bg(Color::WHITE)
    ///                                 .child(text("Hello from modal!"))
    ///                         })
    ///                         .show();
    ///                 }
    ///             })
    ///         )
    /// }
    /// ```
    pub fn overlay_manager(&self) -> OverlayManager {
        Arc::clone(&self.overlay_manager)
    }

    // =========================================================================
    // Query API
    // =========================================================================

    /// Query an element by ID and get an ElementHandle for programmatic manipulation
    ///
    /// Returns an `ElementHandle` for interacting with the element. The handle
    /// provides methods like `scroll_into_view()`, `focus()`, `click()`, `on_ready()`,
    /// and tree traversal.
    ///
    /// The handle works even if the element doesn't exist yet - operations like
    /// `on_ready()` will queue until the element is laid out. Use `handle.exists()`
    /// to check if the element currently exists.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Register on_ready callback (works before element exists):
    /// ctx.query("progress-bar").on_ready(|bounds| {
    ///     progress_anim.lock().unwrap().set_target(bounds.width * 0.75);
    /// });
    ///
    /// // In UI builder:
    /// div().id("progress-bar").child(...)
    ///
    /// // Later, interact with existing element:
    /// let handle = ctx.query("my-element");
    /// if handle.exists() {
    ///     handle.scroll_into_view();
    ///     handle.focus();
    /// }
    /// ```
    pub fn query(&self, id: &str) -> blinc_layout::selector::ElementHandle<()> {
        blinc_layout::selector::ElementHandle::new(id, self.element_registry.clone())
    }

    /// Get the shared element registry
    ///
    /// This provides access to the element registry for advanced query operations.
    pub fn element_registry(&self) -> &SharedElementRegistry {
        &self.element_registry
    }

    /// Create a persistent state for stateful UI elements
    ///
    /// This creates a `SharedState<S>` that survives across UI rebuilds.
    /// State is keyed automatically by source location using `#[track_caller]`.
    ///
    /// Use with `stateful()` for the cleanest API:
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_layout::prelude::*;
    ///
    /// fn my_button(ctx: &WindowedContext) -> impl ElementBuilder {
    ///     let handle = ctx.use_state(ButtonState::Idle);
    ///
    ///     stateful(handle)
    ///         .on_state(|state, div| {
    ///             match state {
    ///                 ButtonState::Hovered => { *div = div.swap().bg(Color::RED); }
    ///                 _ => { *div = div.swap().bg(Color::BLUE); }
    ///             }
    ///         })
    /// }
    /// ```
    #[track_caller]
    pub fn use_state<S>(&self, initial: S) -> blinc_layout::SharedState<S>
    where
        S: blinc_layout::StateTransitions + Clone + Send + 'static,
    {
        // Use caller location as the key
        let location = std::panic::Location::caller();
        let key = format!(
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        self.use_state_for(&key, initial)
    }

    /// Create a persistent state with an explicit key
    ///
    /// Use this for reusable components that are called multiple times
    /// from the same location (e.g., in a loop or when the same component
    /// function is called multiple times with different props).
    ///
    /// The key can be any type that implements `Hash` (strings, numbers, etc).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Reusable component - string key
    /// fn feature_card(ctx: &WindowedContext, id: &str) -> impl ElementBuilder {
    ///     let handle = ctx.use_state_for(id, ButtonState::Idle);
    ///     stateful(handle).on_state(|state, div| { ... })
    /// }
    ///
    /// // Or with numeric key in a loop
    /// for i in 0..3 {
    ///     let handle = ctx.use_state_for(i, ButtonState::Idle);
    ///     // ...
    /// }
    /// ```
    pub fn use_state_for<K, S>(&self, key: K, initial: S) -> blinc_layout::SharedState<S>
    where
        K: Hash,
        S: blinc_layout::StateTransitions + Clone + Send + 'static,
    {
        use blinc_core::reactive::SignalId;
        use blinc_layout::stateful::StatefulInner;

        // We store the SharedState<S> as a signal value
        let state_key = StateKey::new::<blinc_layout::SharedState<S>, _>(&key);
        let mut hooks = self.hooks.lock().unwrap();

        if let Some(raw_id) = hooks.get(&state_key) {
            // Existing state - get the SharedState from the signal
            let signal_id = SignalId::from_raw(raw_id);
            let signal: Signal<blinc_layout::SharedState<S>> = Signal::from_id(signal_id);
            self.reactive.lock().unwrap().get(signal).unwrap()
        } else {
            // New state - create SharedState and store in signal
            let shared_state: blinc_layout::SharedState<S> =
                Arc::new(Mutex::new(StatefulInner::new(initial)));
            let signal = self
                .reactive
                .lock()
                .unwrap()
                .create_signal(shared_state.clone());
            let raw_id = signal.id().to_raw();
            hooks.insert(state_key, raw_id);
            shared_state
        }
    }

    /// Create a persistent animated value using caller location as key
    ///
    /// The animated value survives UI rebuilds, preserving its current value
    /// and active spring animations. This is essential for continuous animations
    /// driven by state changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Animated value persists across rebuilds
    /// let offset_y = ctx.use_animated_value(0.0, SpringConfig::wobbly());
    ///
    /// // Can be used in motion bindings
    /// motion().translate_y(offset_y.clone()).child(content)
    /// ```
    #[track_caller]
    pub fn use_animated_value(&self, initial: f32, config: SpringConfig) -> SharedAnimatedValue {
        let location = std::panic::Location::caller();
        let key = format!(
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        self.use_animated_value_for(&key, initial, config)
    }

    /// Create a persistent animated value with an explicit key
    ///
    /// Use this for reusable components or when creating multiple animated
    /// values at the same source location (e.g., in a loop).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiple animated values with unique keys
    /// for i in 0..3 {
    ///     let scale = ctx.use_animated_value_for(
    ///         format!("item_{}_scale", i),
    ///         1.0,
    ///         SpringConfig::snappy(),
    ///     );
    /// }
    /// ```
    pub fn use_animated_value_for<K: Hash>(
        &self,
        key: K,
        initial: f32,
        config: SpringConfig,
    ) -> SharedAnimatedValue {
        use blinc_core::reactive::SignalId;

        // Use a type marker for SharedAnimatedValue
        let state_key = StateKey::new::<SharedAnimatedValue, _>(&key);
        let mut hooks = self.hooks.lock().unwrap();

        if let Some(raw_id) = hooks.get(&state_key) {
            // Existing animated value - retrieve from signal
            let signal_id = SignalId::from_raw(raw_id);
            let signal: Signal<SharedAnimatedValue> = Signal::from_id(signal_id);
            self.reactive.lock().unwrap().get(signal).unwrap()
        } else {
            // New animated value - create and store in signal
            let animated_value: SharedAnimatedValue = Arc::new(Mutex::new(AnimatedValue::new(
                self.animation_handle(),
                initial,
                config,
            )));
            let signal = self
                .reactive
                .lock()
                .unwrap()
                .create_signal(animated_value.clone());
            let raw_id = signal.id().to_raw();
            hooks.insert(state_key, raw_id);
            animated_value
        }
    }

    /// Create or retrieve a persistent animated timeline
    ///
    /// AnimatedTimeline provides keyframe-based animations that persist across
    /// UI rebuilds. Use this for timeline animations that need to survive
    /// layout changes and window resizes.
    ///
    /// The returned timeline is empty on first call - add keyframes using
    /// `timeline.add()` then call `start()`. Use `has_entries()` to check
    /// if the timeline needs configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let timeline = ctx.use_animated_timeline();
    /// let entry_id = {
    ///     let mut t = timeline.lock().unwrap();
    ///     if !t.has_entries() {
    ///         let id = t.add(0, 2000, 0.0, 1.0);
    ///         t.start();
    ///         id
    ///     } else {
    ///         t.entry_ids().first().copied().unwrap()
    ///     }
    /// };
    /// ```
    #[track_caller]
    pub fn use_animated_timeline(&self) -> SharedAnimatedTimeline {
        let location = std::panic::Location::caller();
        let key = format!(
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        self.use_animated_timeline_for(&key)
    }

    /// Create or retrieve a persistent animated timeline with an explicit key
    ///
    /// Use this for reusable components or when creating multiple timelines
    /// at the same source location (e.g., in a loop).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiple timelines with unique keys
    /// for i in 0..3 {
    ///     let timeline = ctx.use_animated_timeline_for(format!("dot_{}", i));
    ///     // ...
    /// }
    /// ```
    pub fn use_animated_timeline_for<K: Hash>(&self, key: K) -> SharedAnimatedTimeline {
        use blinc_core::reactive::SignalId;

        // Use a type marker for SharedAnimatedTimeline
        let state_key = StateKey::new::<SharedAnimatedTimeline, _>(&key);
        let mut hooks = self.hooks.lock().unwrap();

        if let Some(raw_id) = hooks.get(&state_key) {
            // Existing timeline - retrieve from signal
            let signal_id = SignalId::from_raw(raw_id);
            let signal: Signal<SharedAnimatedTimeline> = Signal::from_id(signal_id);
            self.reactive.lock().unwrap().get(signal).unwrap()
        } else {
            // New timeline - create and store in signal
            let timeline: SharedAnimatedTimeline =
                Arc::new(Mutex::new(AnimatedTimeline::new(self.animation_handle())));
            let signal = self
                .reactive
                .lock()
                .unwrap()
                .create_signal(timeline.clone());
            let raw_id = signal.id().to_raw();
            hooks.insert(state_key, raw_id);
            timeline
        }
    }

    // =========================================================================
    // Tick Callback API (for per-frame updates like ECS systems)
    // =========================================================================

    /// Register a callback that runs each frame in the animation scheduler
    ///
    /// This creates a persistent tick callback keyed by source location.
    /// The callback receives delta time in seconds and runs on the animation
    /// scheduler's background thread at 120fps.
    ///
    /// Use this for ECS systems, physics, or any per-frame updates.
    /// The callback is registered once and persists across UI rebuilds.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Create ECS world (persisted via use_state)
    /// let world = ctx.use_state_keyed("world", || Arc::new(Mutex::new(World::new())));
    ///
    /// // Register tick callback to run ECS systems
    /// ctx.use_tick_callback({
    ///     let world = world.get();
    ///     move |dt| {
    ///         let mut w = world.lock().unwrap();
    ///         // Run ECS systems with delta time
    ///         w.run_systems(dt);
    ///     }
    /// });
    /// ```
    #[track_caller]
    pub fn use_tick_callback<F>(&self, callback: F) -> blinc_animation::TickCallbackId
    where
        F: FnMut(f32) + Send + Sync + 'static,
    {
        let location = std::panic::Location::caller();
        let key = format!(
            "tick_{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        self.use_tick_callback_for(&key, callback)
    }

    /// Register a tick callback with an explicit key
    ///
    /// Use this when you need to create multiple tick callbacks at the same
    /// source location (e.g., in a loop) or in reusable components.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiple tick callbacks with unique keys
    /// for i in 0..3 {
    ///     ctx.use_tick_callback_for(format!("system_{}", i), move |dt| {
    ///         // Per-frame update
    ///     });
    /// }
    /// ```
    pub fn use_tick_callback_for<K: Hash, F>(
        &self,
        key: K,
        callback: F,
    ) -> blinc_animation::TickCallbackId
    where
        F: FnMut(f32) + Send + Sync + 'static,
    {
        // Marker type for TickCallbackId storage
        struct TickCallbackMarker;

        let state_key = StateKey::new::<TickCallbackMarker, _>(&key);
        let mut hooks = self.hooks.lock().unwrap();

        if let Some(raw_id) = hooks.get(&state_key) {
            // Already registered - return existing ID
            blinc_animation::TickCallbackId::from_raw(raw_id)
        } else {
            // First time - register the callback with the scheduler
            let id = self
                .animation_handle()
                .register_tick_callback(callback)
                .expect("Animation scheduler should be alive");
            hooks.insert(state_key, id.to_raw());
            id
        }
    }

    // =========================================================================
    // Theme API
    // =========================================================================

    /// Get the current color scheme (light or dark)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let scheme = ctx.color_scheme();
    /// match scheme {
    ///     ColorScheme::Light => println!("Light mode"),
    ///     ColorScheme::Dark => println!("Dark mode"),
    /// }
    /// ```
    pub fn color_scheme(&self) -> blinc_theme::ColorScheme {
        blinc_theme::ThemeState::get().scheme()
    }

    /// Set the color scheme (triggers smooth theme transition)
    ///
    /// # Example
    ///
    /// ```ignore
    /// ctx.set_color_scheme(ColorScheme::Dark);
    /// ```
    pub fn set_color_scheme(&self, scheme: blinc_theme::ColorScheme) {
        blinc_theme::ThemeState::get().set_scheme(scheme);
    }

    /// Toggle between light and dark mode
    ///
    /// # Example
    ///
    /// ```ignore
    /// button("Toggle Theme").on_click(|ctx| {
    ///     ctx.toggle_color_scheme();
    /// })
    /// ```
    pub fn toggle_color_scheme(&self) {
        blinc_theme::ThemeState::get().toggle_scheme();
    }

    /// Get a color from the current theme
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_theme::ColorToken;
    ///
    /// let primary = ctx.theme_color(ColorToken::Primary);
    /// let bg = ctx.theme_color(ColorToken::Background);
    /// ```
    pub fn theme_color(&self, token: blinc_theme::ColorToken) -> blinc_core::Color {
        blinc_theme::ThemeState::get().color(token)
    }

    /// Get spacing from the current theme
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_theme::SpacingToken;
    ///
    /// let padding = ctx.theme_spacing(SpacingToken::Space4); // 16px
    /// ```
    pub fn theme_spacing(&self, token: blinc_theme::SpacingToken) -> f32 {
        blinc_theme::ThemeState::get().spacing_value(token)
    }

    /// Get border radius from the current theme
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_theme::RadiusToken;
    ///
    /// let radius = ctx.theme_radius(RadiusToken::Lg); // 8px
    /// ```
    pub fn theme_radius(&self, token: blinc_theme::RadiusToken) -> f32 {
        blinc_theme::ThemeState::get().radius(token)
    }

    // =========================================================================
    // i18n API
    // =========================================================================

    /// Get the current locale identifier (e.g., "en-US", "ko-KR")
    pub fn locale(&self) -> String {
        blinc_i18n::I18nState::get().locale()
    }

    /// Set the current locale (triggers a full UI rebuild via i18n redraw callback)
    pub fn set_locale(&self, locale: impl Into<String>) {
        blinc_i18n::I18nState::get().set_locale(locale);
    }

    // =========================================================================
    // CSS Stylesheet API
    // =========================================================================

    /// Add inline CSS to the application stylesheet.
    ///
    /// Multiple calls cascade — later rules override earlier ones.
    /// Stylesheets are visual-only: they update render props on existing nodes
    /// and trigger redraws. They never cause tree rebuilds.
    pub fn add_css(&mut self, css: &str) {
        match blinc_layout::css_parser::Stylesheet::parse(css) {
            Ok(sheet) => self.add_stylesheet(sheet),
            Err(e) => {
                tracing::warn!("Failed to parse CSS: {}", e);
            }
        }
    }

    /// Load and add a `.css` file to the application stylesheet.
    ///
    /// Multiple calls cascade — later rules override earlier ones.
    pub fn load_css(&mut self, path: &str) {
        match blinc_layout::css_parser::Stylesheet::from_file(path) {
            Ok(sheet) => self.add_stylesheet(sheet),
            Err(e) => {
                tracing::warn!("Failed to load CSS file '{}': {}", path, e);
            }
        }
    }

    /// Add a pre-parsed stylesheet to the application.
    ///
    /// Multiple calls cascade — later rules override earlier ones.
    pub fn add_stylesheet(&mut self, sheet: blinc_layout::css_parser::Stylesheet) {
        match self.stylesheet.as_mut() {
            Some(existing) => {
                // Cascade: merge into existing (Arc::make_mut for COW)
                Arc::make_mut(existing).merge(sheet);
            }
            None => {
                self.stylesheet = Some(Arc::new(sheet));
            }
        }
    }

    /// Set a style for an element by ID.
    ///
    /// This is the Rust-native alternative to `add_css()`. Use with `css!` or `style!`
    /// macros to define styles in Rust syntax that are applied automatically to matching
    /// elements — just like CSS stylesheets.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ctx.set_style("card", css! {
    ///     background: Color::BLUE;
    ///     border-radius: 12.0;
    ///     box-shadow: md;
    /// });
    ///
    /// // Then just give the element an ID:
    /// div().id("card").w(200.0).h(100.0)
    /// ```
    pub fn set_style(&mut self, id: &str, style: blinc_layout::element_style::ElementStyle) {
        match self.stylesheet.as_mut() {
            Some(existing) => {
                Arc::make_mut(existing).insert(id, style);
            }
            None => {
                let mut sheet = blinc_layout::css_parser::Stylesheet::new();
                sheet.insert(id, style);
                self.stylesheet = Some(Arc::new(sheet));
            }
        }
    }

    /// Set a state-specific style for an element by ID.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_layout::css_parser::ElementState;
    ///
    /// ctx.set_style("button", style! { bg: Color::BLUE, rounded: 8.0 });
    /// ctx.set_state_style("button", ElementState::Hover, style! {
    ///     bg: Color::from_hex(0x2563EB),
    ///     shadow_md,
    /// });
    /// ```
    pub fn set_state_style(
        &mut self,
        id: &str,
        state: blinc_layout::css_parser::ElementState,
        style: blinc_layout::element_style::ElementStyle,
    ) {
        match self.stylesheet.as_mut() {
            Some(existing) => {
                Arc::make_mut(existing).insert_with_state(id, state, style);
            }
            None => {
                let mut sheet = blinc_layout::css_parser::Stylesheet::new();
                sheet.insert_with_state(id, state, style);
                self.stylesheet = Some(Arc::new(sheet));
            }
        }
    }
}

// =============================================================================
// BlincContext Implementation
// =============================================================================

impl blinc_core::BlincContext for WindowedContext {
    fn use_state_keyed<T, F>(&self, key: &str, init: F) -> State<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        // Delegate to the existing method
        WindowedContext::use_state_keyed(self, key, init)
    }

    fn use_signal_keyed<T, F>(&self, key: &str, init: F) -> Signal<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        WindowedContext::use_signal_keyed(self, key, init)
    }

    fn use_signal<T: Send + 'static>(&self, initial: T) -> Signal<T> {
        WindowedContext::use_signal(self, initial)
    }

    fn get<T: Clone + 'static>(&self, signal: Signal<T>) -> Option<T> {
        WindowedContext::get(self, signal)
    }

    fn set<T: Send + 'static>(&self, signal: Signal<T>, value: T) {
        WindowedContext::set(self, signal, value)
    }

    fn update<T: Clone + Send + 'static, F: FnOnce(T) -> T>(&self, signal: Signal<T>, f: F) {
        WindowedContext::update(self, signal, f)
    }

    fn use_derived<T, F>(&self, compute: F) -> Derived<T>
    where
        T: Clone + Send + 'static,
        F: Fn(&ReactiveGraph) -> T + Send + 'static,
    {
        WindowedContext::use_derived(self, compute)
    }

    fn get_derived<T: Clone + 'static>(&self, derived: Derived<T>) -> Option<T> {
        WindowedContext::get_derived(self, derived)
    }

    fn batch<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ReactiveGraph) -> R,
    {
        WindowedContext::batch(self, f)
    }

    fn dirty_flag(&self) -> blinc_core::DirtyFlag {
        WindowedContext::dirty_flag(self)
    }

    fn request_rebuild(&self) {
        self.ref_dirty_flag.store(true, Ordering::SeqCst);
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}

// =============================================================================
// AnimationContext Implementation
// =============================================================================

impl AnimationContext for WindowedContext {
    fn animation_handle(&self) -> SchedulerHandle {
        WindowedContext::animation_handle(self)
    }

    fn use_animated_value_for<K: Hash>(
        &self,
        key: K,
        initial: f32,
        config: SpringConfig,
    ) -> SharedAnimatedValue {
        WindowedContext::use_animated_value_for(self, key, initial, config)
    }

    fn use_animated_timeline_for<K: Hash>(&self, key: K) -> SharedAnimatedTimeline {
        WindowedContext::use_animated_timeline_for(self, key)
    }
}

/// Windowed application runner
///
/// Provides a simple way to run a Blinc application in a window
/// with automatic event handling and rendering.
pub struct WindowedApp;

impl WindowedApp {
    /// Initialize the platform asset loader
    ///
    /// On desktop, this sets up a filesystem-based loader.
    /// On Android, this would use the NDK AssetManager.
    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    fn init_asset_loader() {
        use blinc_platform::assets::{set_global_asset_loader, FilesystemAssetLoader};

        // Create a filesystem loader (uses current directory as base)
        let loader = FilesystemAssetLoader::new();

        // Try to set the global loader (ignore error if already set)
        let _ = set_global_asset_loader(Box::new(loader));
    }

    /// Initialize the theme system with platform detection
    ///
    /// This sets up the global ThemeState with:
    /// - Platform-appropriate theme bundle (macOS, Windows, Linux, etc.)
    /// - System color scheme detection (light/dark mode)
    /// - Redraw callback to trigger UI updates on theme changes
    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    fn init_theme() {
        use blinc_theme::{
            detect_system_color_scheme, platform_theme_bundle, set_redraw_callback, ThemeState,
        };

        // Only initialize if not already initialized
        if ThemeState::try_get().is_none() {
            let bundle = platform_theme_bundle();
            let scheme = detect_system_color_scheme();
            ThemeState::init(bundle, scheme);
        }

        // Set up the redraw callback to trigger full UI rebuilds when theme changes
        // We use request_full_rebuild() to trigger all three phases:
        // 1. Tree rebuild - reconstruct UI with new theme values
        // 2. Layout recompute - recalculate flexbox layout
        // 3. Visual redraw - render the frame
        set_redraw_callback(|| {
            tracing::debug!("Theme changed - requesting full rebuild");
            blinc_layout::widgets::request_full_rebuild();
        });
    }

    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    fn init_i18n() {
        use blinc_i18n::I18nState;

        fn parse_env_locale(raw: &str) -> Option<String> {
            // Examples: "en_US.UTF-8", "ko_KR", "en-US", "C"
            let s = raw.trim();
            let s = s.split_once('.').map_or(s, |(part, _)| part);
            let s = s.split_once('@').map_or(s, |(part, _)| part);
            let s = s.trim();
            if s.is_empty() || s == "C" || s == "POSIX" {
                return None;
            }
            Some(s.replace('_', "-"))
        }

        fn detect_locale_from_env() -> Option<String> {
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .iter()
                .find_map(|key| std::env::var(key).ok().and_then(|v| parse_env_locale(&v)))
        }

        // Only initialize if not already initialized
        if I18nState::try_get().is_none() {
            let locale = detect_locale_from_env().unwrap_or_else(|| "en-US".to_string());
            I18nState::init(locale);
        }

        // Trigger full rebuild on locale changes (same behavior as theme changes).
        blinc_i18n::set_redraw_callback(|| {
            tracing::debug!("Locale changed - requesting full rebuild");
            blinc_layout::widgets::request_full_rebuild();
        });
    }

    /// Run a windowed Blinc application on desktop platforms
    ///
    /// This is the main entry point for desktop applications. It creates
    /// a window, sets up GPU rendering, and runs the event loop.
    ///
    /// # Arguments
    ///
    /// * `config` - Window configuration (title, size, etc.)
    /// * `ui_builder` - Function that builds the UI tree given the window context
    ///
    /// # Example
    ///
    /// ```ignore
    /// WindowedApp::run(WindowConfig::default(), |ctx| {
    ///     div()
    ///         .w(ctx.width).h(ctx.height)
    ///         .bg([0.1, 0.1, 0.15, 1.0])
    ///         .flex_center()
    ///         .child(
    ///             div().glass().rounded(16.0).p(24.0)
    ///                 .child(text("Hello Blinc!").size(32.0))
    ///         )
    /// })
    /// ```
    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    pub fn run<F, E>(config: WindowConfig, ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder + 'static,
    {
        Self::run_desktop(config, ui_builder)
    }

    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    fn run_desktop<F, E>(config: WindowConfig, mut ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder + 'static,
    {
        // Initialize the platform asset loader for cross-platform asset loading
        Self::init_asset_loader();

        // Initialize the text measurer for accurate text layout
        crate::text_measurer::init_text_measurer();

        // Initialize the theme system with platform detection
        Self::init_theme();

        // Initialize i18n (locale + redraw hook)
        Self::init_i18n();

        let platform = DesktopPlatform::new().map_err(|e| BlincError::Platform(e.to_string()))?;
        let event_loop = platform
            .create_event_loop_with_config(config)
            .map_err(|e| BlincError::Platform(e.to_string()))?;

        // Get a wake proxy to allow the animation thread to wake up the event loop
        let wake_proxy = event_loop.wake_proxy();

        // We need to defer BlincApp creation until we have a window
        let mut app: Option<BlincApp> = None;
        let mut surface: Option<wgpu::Surface<'static>> = None;
        let mut surface_config: Option<wgpu::SurfaceConfiguration> = None;

        // Persistent context with event router
        let mut ctx: Option<WindowedContext> = None;
        // Persistent render tree for hit testing and dirty tracking
        let mut render_tree: Option<RenderTree> = None;
        // Track last frame time for CSS animation delta calculation
        let mut last_frame_time_ms: u64 = 0;
        // Track if we need to rebuild UI (e.g., after resize)
        let mut needs_rebuild = true;
        // Track if we need to relayout (e.g., after resize even if tree unchanged)
        let mut needs_relayout = false;
        // Shared dirty flag for element refs
        let ref_dirty_flag: RefDirtyFlag = Arc::new(AtomicBool::new(false));
        // Shared reactive graph for signal-based state management
        let reactive: SharedReactiveGraph = Arc::new(Mutex::new(ReactiveGraph::new()));
        // Shared hook state for use_state persistence
        let hooks: SharedHookState = Arc::new(Mutex::new(HookState::new()));

        // Initialize global context state singleton (if not already initialized)
        // This allows components to create internal state without context parameters
        if !BlincContextState::is_initialized() {
            #[allow(clippy::type_complexity)]
            let stateful_callback: std::sync::Arc<dyn Fn(&[SignalId]) + Send + Sync> =
                Arc::new(|signal_ids| {
                    blinc_layout::check_stateful_deps(signal_ids);
                });
            BlincContextState::init_with_callback(
                Arc::clone(&reactive),
                Arc::clone(&hooks),
                Arc::clone(&ref_dirty_flag),
                stateful_callback,
            );
        }

        // Shared animation scheduler for spring/keyframe animations
        // Runs on background thread so animations continue even when window loses focus
        let mut scheduler = AnimationScheduler::new();
        // Set up wake callback so animation thread can wake the event loop
        scheduler.set_wake_callback(move || wake_proxy.wake());
        scheduler.start_background();
        let animations: SharedAnimationScheduler = Arc::new(Mutex::new(scheduler));

        // Set global scheduler handle for StateContext and component access
        {
            let scheduler_handle = animations.lock().unwrap().handle();
            blinc_animation::set_global_scheduler(scheduler_handle);
        }

        // Shared CSS animation/transition store
        // The scheduler's background thread keeps the redraw loop alive at 120fps
        // via the tick callback below (acts as keep-alive signal). Actual ticking
        // happens synchronously on the main thread to avoid phase jitter between
        // the bg thread's tick timing and the frame's render timing.
        let css_anim_store = Arc::new(Mutex::new(blinc_layout::CssAnimationStore::new()));
        {
            animations
                .lock()
                .unwrap()
                .add_tick_callback(move |_dt_secs| {
                    // Keep-alive no-op: the callback's existence ensures the scheduler
                    // considers tick_callbacks as "active", triggering wake_callback()
                    // at 120fps so the main thread gets continuous frame requests
                    // while CSS animations are running.
                    // Actual ticking happens on the main thread to avoid phase jitter.
                });
        }

        // Shared element registry for query API
        let element_registry: SharedElementRegistry =
            Arc::new(blinc_layout::selector::ElementRegistry::new());

        // Set up query callback in BlincContextState so components can query elements globally
        {
            let registry_for_query = Arc::clone(&element_registry);
            let query_callback: blinc_core::QueryCallback = Arc::new(move |id: &str| {
                registry_for_query.get(id).map(|node_id| node_id.to_raw())
            });
            BlincContextState::get().set_query_callback(query_callback);
        }

        // Set up bounds callback for ElementHandle.bounds()
        {
            let registry_for_bounds = Arc::clone(&element_registry);
            let bounds_callback: blinc_core::BoundsCallback =
                Arc::new(move |id: &str| registry_for_bounds.get_bounds(id));
            BlincContextState::get().set_bounds_callback(bounds_callback);
        }

        // Store element registry in BlincContextState for global query() function
        // Cast to Arc<dyn Any + Send + Sync> for type-erased storage
        BlincContextState::get()
            .set_element_registry(Arc::clone(&element_registry) as blinc_core::AnyElementRegistry);

        // Shared storage for on_ready callbacks
        let ready_callbacks: SharedReadyCallbacks = Arc::new(Mutex::new(Vec::new()));

        // Set up continuous redraw callback for text widget cursor animation
        // This bridges text widgets (which track focus) with the animation scheduler (which drives redraws)
        {
            let animations_for_callback = Arc::clone(&animations);
            blinc_layout::widgets::set_continuous_redraw_callback(move |enabled| {
                if let Ok(scheduler) = animations_for_callback.lock() {
                    scheduler.set_continuous_redraw(enabled);
                }
            });
        }

        // Connect theme animation to the animation scheduler
        // This enables smooth color transitions when switching between light/dark mode
        blinc_theme::ThemeState::get().set_scheduler(&animations);

        // Render state: dynamic properties that update every frame without tree rebuild
        // This includes cursor blink, animated colors, hover states, etc.
        let mut render_state: Option<blinc_layout::RenderState> = None;

        // Shared motion states for query API access
        // This allows components to query motion animation state via query_motion()
        let shared_motion_states = blinc_layout::create_shared_motion_states();

        // Set up motion state callback in BlincContextState
        {
            let motion_states_for_callback = Arc::clone(&shared_motion_states);
            let motion_callback: blinc_core::MotionStateCallback = Arc::new(move |key: &str| {
                motion_states_for_callback
                    .read()
                    .ok()
                    .and_then(|states| states.get(key).copied())
                    .unwrap_or(blinc_core::MotionAnimationState::NotFound)
            });
            BlincContextState::get().set_motion_state_callback(motion_callback);
        }

        // Overlay manager for modals, dialogs, toasts, etc.
        let overlays: OverlayManager = overlay_manager();

        // Initialize overlay context singleton for component access
        if !OverlayContext::is_initialized() {
            OverlayContext::init(Arc::clone(&overlays));
        }

        let e2e_enabled = e2e_is_enabled();
        let e2e_expect = e2e_expect();
        let e2e_capture_path = std::env::var("BLINC_E2E_CAPTURE_PATH")
            .ok()
            .map(std::path::PathBuf::from);
        let e2e_trigger_path = e2e_trigger_path();
        let e2e_capture_on_start = e2e_capture_on_start(e2e_trigger_path.as_ref());
        let e2e_max_captures = e2e_max_captures();
        let e2e_exit = e2e_exit_after();
        let mut e2e_captures_done: usize = 0;

        let e2e_script = e2e_script();
        let e2e_script_exit = e2e_script_exit_after(e2e_script.is_some());
        let mut e2e_script_ran: bool = false;
        #[cfg(feature = "webview")]
        let mut webview_lifecycle = DesktopWebViewLifecycle::default();

        event_loop
            .run(move |event, window| {
                match event {
                    Event::Lifecycle(LifecycleEvent::Resumed) => {
                        // Initialize GPU if not already done
                        if app.is_none() {
                            let winit_window = window.winit_window_arc();

                            match BlincApp::with_window(winit_window, None) {
                                Ok((blinc_app, surf)) => {
                                    let (width, height) = window.size();
                                    // Use the same texture format that the renderer's pipelines use
                                    let format = blinc_app.texture_format();
                                    let config = wgpu::SurfaceConfiguration {
                                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                            | if e2e_enabled {
                                                wgpu::TextureUsages::COPY_SRC
                                            } else {
                                                wgpu::TextureUsages::empty()
                                            },
                                        format,
                                        width,
                                        height,
                                        present_mode: wgpu::PresentMode::AutoVsync,
                                        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                                        view_formats: vec![],
                                        desired_maximum_frame_latency: 2,
                                    };
                                    surf.configure(blinc_app.device(), &config);

                                    // Update text measurer with shared font registry for accurate measurement
                                    crate::text_measurer::init_text_measurer_with_registry(
                                        blinc_app.font_registry(),
                                    );

                                    surface = Some(surf);
                                    surface_config = Some(config);
                                    app = Some(blinc_app);

                                    // Initialize context with event router, animations, dirty flag, reactive graph, hooks, overlay manager, registry, and ready callbacks
                                    ctx = Some(WindowedContext::from_window(
                                        window,
                                        EventRouter::new(),
                                        Arc::clone(&animations),
                                        Arc::clone(&ref_dirty_flag),
                                        Arc::clone(&reactive),
                                        Arc::clone(&hooks),
                                        Arc::clone(&overlays),
                                        Arc::clone(&element_registry),
                                        Arc::clone(&ready_callbacks),
                                    ));

                                    // Set initial viewport size in BlincContextState
                                    if let Some(ref windowed_ctx) = ctx {
                                        BlincContextState::get().set_viewport_size(windowed_ctx.width, windowed_ctx.height);
                                    }

                                    // Initialize render state with the shared animation scheduler
                                    // RenderState handles dynamic properties (cursor blink, animations)
                                    // independently from tree structure changes
                                    let mut rs = blinc_layout::RenderState::new(Arc::clone(&animations));
                                    rs.set_shared_motion_states(Arc::clone(&shared_motion_states));
                                    render_state = Some(rs);

                                    tracing::debug!("Blinc windowed app initialized");
                                }
                                Err(e) => {
                                    tracing::error!("Failed to initialize Blinc: {}", e);
                                    #[cfg(feature = "webview")]
                                    webview_lifecycle.dispose();
                                    return ControlFlow::Exit;
                                }
                            }
                        }

                        #[cfg(feature = "webview")]
                        {
                            webview_lifecycle.ensure_created(window);
                            webview_lifecycle.sync_bounds(window);
                            if let Some(ref windowed_ctx) = ctx {
                                webview_lifecycle.on_focus_changed(windowed_ctx.focused);
                            }
                        }
                    }

                    Event::Lifecycle(LifecycleEvent::Suspended) => {
                        #[cfg(feature = "webview")]
                        webview_lifecycle.dispose();
                    }

                    Event::Window(WindowEvent::Resized { width, height }) => {
                        if let (Some(ref blinc_app), Some(ref surf), Some(ref mut config)) =
                            (&app, &surface, &mut surface_config)
                        {
                            if width > 0 && height > 0 {
                                config.width = width;
                                config.height = height;
                                surf.configure(blinc_app.device(), config);
                                needs_rebuild = true;
                                needs_relayout = true;

                                // Dispatch RESIZE event to elements (use logical dimensions)
                                if let (Some(ref mut windowed_ctx), Some(ref tree)) =
                                    (&mut ctx, &render_tree)
                                {
                                    let logical_width = width as f32 / windowed_ctx.scale_factor as f32;
                                    let logical_height = height as f32 / windowed_ctx.scale_factor as f32;

                                    // Update windowed context dimensions - CRITICAL for layout computation
                                    // Without this, compute_layout uses stale dimensions
                                    windowed_ctx.width = logical_width;
                                    windowed_ctx.height = logical_height;
                                    windowed_ctx.physical_width = width as f32;
                                    windowed_ctx.physical_height = height as f32;

                                    // Update viewport size in BlincContextState for ElementHandle.is_visible()
                                    BlincContextState::get().set_viewport_size(logical_width, logical_height);

                                    windowed_ctx
                                        .event_router
                                        .on_window_resize(tree, logical_width, logical_height);

                                    // Clear layout bounds storages to force fresh calculations
                                    // This prevents stale cached bounds from influencing the new layout
                                    tree.clear_layout_bounds_storages();
                                }

                                // Request redraw to trigger relayout with new dimensions
                                window.request_redraw();

                                #[cfg(feature = "webview")]
                                webview_lifecycle.sync_bounds(window);
                            }
                        }
                    }

                    Event::Window(WindowEvent::ScaleFactorChanged { scale_factor }) => {
                        needs_rebuild = true;
                        needs_relayout = true;

                        let (width, height) = window.size();
                        if let (Some(ref mut windowed_ctx), Some(ref tree)) = (&mut ctx, &render_tree) {
                            let logical_width = width as f32 / scale_factor as f32;
                            let logical_height = height as f32 / scale_factor as f32;

                            windowed_ctx.width = logical_width;
                            windowed_ctx.height = logical_height;
                            windowed_ctx.scale_factor = scale_factor;
                            windowed_ctx.physical_width = width as f32;
                            windowed_ctx.physical_height = height as f32;

                            BlincContextState::get().set_viewport_size(logical_width, logical_height);
                            windowed_ctx
                                .event_router
                                .on_window_resize(tree, logical_width, logical_height);
                            tree.clear_layout_bounds_storages();
                        }

                        #[cfg(feature = "webview")]
                        webview_lifecycle.sync_bounds(window);

                        window.request_redraw();
                    }

                    Event::Window(WindowEvent::Focused(focused)) => {
                        // Update context focus state
                        if let Some(ref mut windowed_ctx) = ctx {
                            windowed_ctx.focused = focused;

                            // Dispatch WINDOW_FOCUS or WINDOW_BLUR to the focused element
                            windowed_ctx.event_router.on_window_focus(focused);

                            // When window loses focus, blur all text inputs/areas
                            if !focused {
                                blinc_layout::widgets::blur_all_text_inputs();
                            }
                        }

                        #[cfg(feature = "webview")]
                        webview_lifecycle.on_focus_changed(focused);
                    }

                    Event::Window(WindowEvent::CloseRequested) => {
                        #[cfg(feature = "webview")]
                        webview_lifecycle.dispose();
                        return ControlFlow::Exit;
                    }

                    // Handle input events
                    Event::Input(input_event) => {
                        // Pending event structure for deferred dispatch
                        #[derive(Clone)]
                        struct PendingEvent {
                            node_id: LayoutNodeId,
                            event_type: u32,
                            mouse_x: f32,
                            mouse_y: f32,
                            /// Local coordinates relative to element bounds
                            local_x: f32,
                            local_y: f32,
                            /// Absolute position of element bounds (top-left corner)
                            bounds_x: f32,
                            bounds_y: f32,
                            /// Computed bounds dimensions of the element
                            bounds_width: f32,
                            bounds_height: f32,
                            /// Drag delta for DRAG/DRAG_END events
                            drag_delta_x: f32,
                            drag_delta_y: f32,
                            key_char: Option<char>,
                            key_code: u32,
                            shift: bool,
                            ctrl: bool,
                            alt: bool,
                            meta: bool,
                        }

                        impl Default for PendingEvent {
                            fn default() -> Self {
                                Self {
                                    node_id: LayoutNodeId::default(),
                                    event_type: 0,
                                    mouse_x: 0.0,
                                    mouse_y: 0.0,
                                    local_x: 0.0,
                                    local_y: 0.0,
                                    bounds_x: 0.0,
                                    bounds_y: 0.0,
                                    bounds_width: 0.0,
                                    bounds_height: 0.0,
                                    drag_delta_x: 0.0,
                                    drag_delta_y: 0.0,
                                    key_char: None,
                                    key_code: 0,
                                    shift: false,
                                    ctrl: false,
                                    alt: false,
                                    meta: false,
                                }
                            }
                        }

                        // First phase: collect events using immutable borrow
                        let (pending_events, keyboard_events, scroll_ended, gesture_ended, scroll_info, pinch_info) = if let (Some(ref mut windowed_ctx), Some(ref tree)) =
                            (&mut ctx, &render_tree)
                        {
                            let router = &mut windowed_ctx.event_router;

                            // Collect events from router
                            let mut pending_events: Vec<PendingEvent> = Vec::new();
                            // Separate collection for keyboard events (TEXT_INPUT)
                            let mut keyboard_events: Vec<PendingEvent> = Vec::new();
                            // Track if scroll ended (momentum finished)
                            let mut scroll_ended = false;
                            // Track if gesture ended (finger lifted - may still have momentum)
                            let mut gesture_ended = false;
                            // Track scroll info for nested scroll dispatch (mouse_x, mouse_y, delta_x, delta_y)
                            let mut scroll_info: Option<(f32, f32, f32, f32)> = None;
                            // Track pinch (magnify) info for dispatch (mouse_x, mouse_y, scale_ratio_delta)
                            let mut pinch_info: Option<(f32, f32, f32)> = None;

                            // Set up callback to collect events
                            router.set_event_callback({
                                let events = &mut pending_events as *mut Vec<PendingEvent>;
                                move |node, event_type| {
                                    // SAFETY: This callback is only used within this scope
                                    unsafe {
                                        (*events).push(PendingEvent {
                                            node_id: node,
                                            event_type,
                                            ..Default::default()
                                        });
                                    }
                                }
                            });

                            // Note: Overlays are now part of the main tree, so all events
                            // are routed through the single main event router.

                            // Convert physical coordinates to logical for hit testing
                            let scale = windowed_ctx.scale_factor as f32;

                            match input_event {
                                InputEvent::Mouse(mouse_event) => match mouse_event {
                                    MouseEvent::Moved { x, y } => {
                                        // Convert physical to logical coordinates
                                        let lx = x / scale;
                                        let ly = y / scale;

                                        // Get overlay bounds and layer ID for occlusion-aware hit testing
                                        // This prevents background elements from receiving hover events
                                        // when they are visually occluded by overlay content
                                        let overlay_bounds = windowed_ctx.overlay_manager.get_visible_overlay_bounds();
                                        let overlay_layer_id = tree.query_by_id(
                                            blinc_layout::widgets::overlay::OVERLAY_LAYER_ID
                                        );

                                        // Route mouse move through main tree with overlay occlusion awareness
                                        router.on_mouse_move_with_occlusion(
                                            tree,
                                            lx,
                                            ly,
                                            &overlay_bounds,
                                            overlay_layer_id,
                                        );

                                        // Get drag delta from router (for DRAG events)
                                        let (drag_dx, drag_dy) = router.drag_delta();

                                        // Populate bounds for each event from the router's hit test results
                                        // This is needed for POINTER_ENTER/POINTER_LEAVE/POINTER_MOVE events
                                        for event in pending_events.iter_mut() {
                                            event.mouse_x = lx;
                                            event.mouse_y = ly;
                                            // Populate drag delta for DRAG events
                                            if event.event_type == blinc_core::events::event_types::DRAG
                                                || event.event_type == blinc_core::events::event_types::DRAG_END
                                            {
                                                event.drag_delta_x = drag_dx;
                                                event.drag_delta_y = drag_dy;
                                            }
                                            // Populate bounds from hit test results (stored in router)
                                            if let Some((bx, by, bw, bh)) = router.get_node_bounds(event.node_id) {
                                                event.bounds_x = bx;
                                                event.bounds_y = by;
                                                event.bounds_width = bw;
                                                event.bounds_height = bh;
                                                event.local_x = lx - bx;
                                                event.local_y = ly - by;
                                            }
                                        }

                                        // Update cursor based on hovered element
                                        let cursor = tree
                                            .get_cursor_at(router, lx, ly)
                                            .unwrap_or(CursorStyle::Default);
                                        window.set_cursor(convert_cursor_style(cursor));
                                    }
                                    MouseEvent::ButtonPressed { button, x, y } => {
                                        let lx = x / scale;
                                        let ly = y / scale;
                                        let btn = convert_mouse_button(button);

                                        if std::env::var_os("BLINC_DEBUG_HIT").is_some() {
                                            if let Some(hit) = router.hit_test(tree, lx, ly) {
                                                let id = tree.element_registry().get_id(hit.node);
                                                tracing::info!(
                                                    "debug_hit: down pos=({:.1}, {:.1}) node={:?} id={:?}",
                                                    lx,
                                                    ly,
                                                    hit.node,
                                                    id
                                                );
                                            } else {
                                                tracing::info!(
                                                    "debug_hit: down pos=({:.1}, {:.1}) node=None",
                                                    lx,
                                                    ly
                                                );
                                            }
                                        }

                                        // Check for backdrop clicks (dismisses overlays)
                                        // This still needs special handling because backdrop clicks should
                                        // not propagate to elements behind the overlay
                                        let overlay_dismissed = if windowed_ctx.overlay_manager.has_blocking_overlay()
                                            || windowed_ctx.overlay_manager.has_dismissable_overlay()
                                        {
                                            windowed_ctx.overlay_manager.handle_click_at(lx, ly)
                                        } else {
                                            false
                                        };

                                        // If overlay was dismissed by backdrop click, don't process further
                                        if !overlay_dismissed {
                                            // Blur any focused text inputs BEFORE processing mouse down
                                            // This mimics HTML behavior where clicking anywhere blurs inputs,
                                            // and clicking on an input then re-focuses it via its own handler
                                            blinc_layout::widgets::blur_all_text_inputs();

                                            // Route through main tree (includes overlay content)
                                            let _events = router.on_mouse_down(tree, lx, ly, btn);

                                            let (local_x, local_y) = router.last_hit_local();
                                            let (bounds_x, bounds_y) = router.last_hit_bounds_pos();
                                            let (bounds_width, bounds_height) = router.last_hit_bounds();
                                            for event in pending_events.iter_mut() {
                                                event.mouse_x = lx;
                                                event.mouse_y = ly;
                                                event.local_x = local_x;
                                                event.local_y = local_y;
                                                event.bounds_x = bounds_x;
                                                event.bounds_y = bounds_y;
                                                event.bounds_width = bounds_width;
                                                event.bounds_height = bounds_height;
                                            }
                                        }
                                    }
                                    MouseEvent::ButtonReleased { button, x, y } => {
                                        let lx = x / scale;
                                        let ly = y / scale;
                                        let btn = convert_mouse_button(button);

                                        // Route through main tree (includes overlay content)
                                        router.on_mouse_up(tree, lx, ly, btn);
                                        // Use the local coordinates from when the press started
                                        // (stored by on_mouse_down via last_hit_local)
                                        let (local_x, local_y) = router.last_hit_local();
                                        let (bounds_x, bounds_y) = router.last_hit_bounds_pos();
                                        let (bounds_width, bounds_height) = router.last_hit_bounds();
                                        for event in pending_events.iter_mut() {
                                            event.mouse_x = lx;
                                            event.mouse_y = ly;
                                            event.local_x = local_x;
                                            event.local_y = local_y;
                                            event.bounds_x = bounds_x;
                                            event.bounds_y = bounds_y;
                                            event.bounds_width = bounds_width;
                                            event.bounds_height = bounds_height;
                                        }
                                    }
                                    MouseEvent::Left => {
                                        // on_mouse_leave now emits POINTER_UP if there was a pressed target
                                        // This handles the case where mouse leaves window while dragging
                                        router.on_mouse_leave();
                                        // Reset cursor to default when mouse leaves window
                                        window.set_cursor(blinc_platform::Cursor::Default);
                                        // Events are collected via the callback set above
                                    }
                                    MouseEvent::Entered => {
                                        let (mx, my) = router.mouse_position();

                                        // Use occlusion-aware hit testing when mouse enters window
                                        let overlay_bounds = windowed_ctx.overlay_manager.get_visible_overlay_bounds();
                                        let overlay_layer_id = tree.query_by_id(
                                            blinc_layout::widgets::overlay::OVERLAY_LAYER_ID
                                        );
                                        router.on_mouse_move_with_occlusion(
                                            tree,
                                            mx,
                                            my,
                                            &overlay_bounds,
                                            overlay_layer_id,
                                        );

                                        for event in pending_events.iter_mut() {
                                            event.mouse_x = mx;
                                            event.mouse_y = my;
                                        }

                                        // Update cursor based on hovered element
                                        let cursor = tree
                                            .get_cursor_at(router, mx, my)
                                            .unwrap_or(CursorStyle::Default);
                                        window.set_cursor(convert_cursor_style(cursor));
                                    }
                                },
                                InputEvent::Keyboard(kb_event) => {
                                    let mods = &kb_event.modifiers;

                                    // Extract character from key if applicable
                                    let key_char = match &kb_event.key {
                                        Key::Char(c) => Some(*c),
                                        Key::Space => Some(' '),
                                        Key::A => Some(if mods.shift { 'A' } else { 'a' }),
                                        Key::B => Some(if mods.shift { 'B' } else { 'b' }),
                                        Key::C => Some(if mods.shift { 'C' } else { 'c' }),
                                        Key::D => Some(if mods.shift { 'D' } else { 'd' }),
                                        Key::E => Some(if mods.shift { 'E' } else { 'e' }),
                                        Key::F => Some(if mods.shift { 'F' } else { 'f' }),
                                        Key::G => Some(if mods.shift { 'G' } else { 'g' }),
                                        Key::H => Some(if mods.shift { 'H' } else { 'h' }),
                                        Key::I => Some(if mods.shift { 'I' } else { 'i' }),
                                        Key::J => Some(if mods.shift { 'J' } else { 'j' }),
                                        Key::K => Some(if mods.shift { 'K' } else { 'k' }),
                                        Key::L => Some(if mods.shift { 'L' } else { 'l' }),
                                        Key::M => Some(if mods.shift { 'M' } else { 'm' }),
                                        Key::N => Some(if mods.shift { 'N' } else { 'n' }),
                                        Key::O => Some(if mods.shift { 'O' } else { 'o' }),
                                        Key::P => Some(if mods.shift { 'P' } else { 'p' }),
                                        Key::Q => Some(if mods.shift { 'Q' } else { 'q' }),
                                        Key::R => Some(if mods.shift { 'R' } else { 'r' }),
                                        Key::S => Some(if mods.shift { 'S' } else { 's' }),
                                        Key::T => Some(if mods.shift { 'T' } else { 't' }),
                                        Key::U => Some(if mods.shift { 'U' } else { 'u' }),
                                        Key::V => Some(if mods.shift { 'V' } else { 'v' }),
                                        Key::W => Some(if mods.shift { 'W' } else { 'w' }),
                                        Key::X => Some(if mods.shift { 'X' } else { 'x' }),
                                        Key::Y => Some(if mods.shift { 'Y' } else { 'y' }),
                                        Key::Z => Some(if mods.shift { 'Z' } else { 'z' }),
                                        Key::Num0 => Some(if mods.shift { ')' } else { '0' }),
                                        Key::Num1 => Some(if mods.shift { '!' } else { '1' }),
                                        Key::Num2 => Some(if mods.shift { '@' } else { '2' }),
                                        Key::Num3 => Some(if mods.shift { '#' } else { '3' }),
                                        Key::Num4 => Some(if mods.shift { '$' } else { '4' }),
                                        Key::Num5 => Some(if mods.shift { '%' } else { '5' }),
                                        Key::Num6 => Some(if mods.shift { '^' } else { '6' }),
                                        Key::Num7 => Some(if mods.shift { '&' } else { '7' }),
                                        Key::Num8 => Some(if mods.shift { '*' } else { '8' }),
                                        Key::Num9 => Some(if mods.shift { '(' } else { '9' }),
                                        Key::Minus => Some(if mods.shift { '_' } else { '-' }),
                                        Key::Equals => Some(if mods.shift { '+' } else { '=' }),
                                        Key::LeftBracket => Some(if mods.shift { '{' } else { '[' }),
                                        Key::RightBracket => Some(if mods.shift { '}' } else { ']' }),
                                        Key::Backslash => Some(if mods.shift { '|' } else { '\\' }),
                                        Key::Semicolon => Some(if mods.shift { ':' } else { ';' }),
                                        Key::Quote => Some(if mods.shift { '"' } else { '\'' }),
                                        Key::Comma => Some(if mods.shift { '<' } else { ',' }),
                                        Key::Period => Some(if mods.shift { '>' } else { '.' }),
                                        Key::Slash => Some(if mods.shift { '?' } else { '/' }),
                                        Key::Grave => Some(if mods.shift { '~' } else { '`' }),
                                        _ => None,
                                    };

                                    // Key code for special key handling (backspace, arrows, etc)
                                    let key_code = match &kb_event.key {
                                        Key::Backspace => 8,
                                        Key::Delete => 127,
                                        Key::Enter => 13,
                                        Key::Tab => 9,
                                        Key::Escape => 27,
                                        Key::Left => 37,
                                        Key::Right => 39,
                                        Key::Up => 38,
                                        Key::Down => 40,
                                        Key::Home => 36,
                                        Key::End => 35,
                                        _ => 0,
                                    };

                                    match kb_event.state {
                                        KeyState::Pressed => {
                                            // Handle Escape key for overlays first
                                            // If an overlay handles it, don't propagate further
                                            if kb_event.key == Key::Escape
                                                && windowed_ctx.overlay_manager.handle_escape()
                                            {
                                                // Escape was consumed by overlay, skip further processing
                                                // (but continue collecting events for non-overlay targets)
                                            }

                                            // Dispatch KEY_DOWN for all keys
                                            router.on_key_down_with_modifiers(
                                                key_code,
                                                mods.shift,
                                                mods.ctrl,
                                                mods.alt,
                                                mods.meta,
                                            );

                                            // For character-producing keys, dispatch TEXT_INPUT
                                            // We use broadcast dispatch so any focused text input can receive it
                                            if let Some(c) = key_char {
                                                // Don't send text input if ctrl/cmd is held (shortcuts)
                                                if !mods.ctrl && !mods.meta {
                                                    keyboard_events.push(PendingEvent {
                                                        event_type: blinc_core::events::event_types::TEXT_INPUT,
                                                        key_char: Some(c),
                                                        key_code,
                                                        shift: mods.shift,
                                                        ctrl: mods.ctrl,
                                                        alt: mods.alt,
                                                        meta: mods.meta,
                                                        ..Default::default()
                                                    });
                                                }
                                            }

                                            // For KEY_DOWN events with special keys (backspace, arrows)
                                            if key_code != 0 {
                                                keyboard_events.push(PendingEvent {
                                                    event_type: blinc_core::events::event_types::KEY_DOWN,
                                                    key_char: None,
                                                    key_code,
                                                    shift: mods.shift,
                                                    ctrl: mods.ctrl,
                                                    alt: mods.alt,
                                                    meta: mods.meta,
                                                    ..Default::default()
                                                });
                                            }
                                        }
                                        KeyState::Released => {
                                            router.on_key_up_with_modifiers(
                                                key_code,
                                                mods.shift,
                                                mods.ctrl,
                                                mods.alt,
                                                mods.meta,
                                            );
                                        }
                                    }
                                },
                                InputEvent::Touch(touch_event) => match touch_event {
                                    TouchEvent::Started { x, y, .. } => {
                                        let lx = x / scale;
                                        let ly = y / scale;
                                        router.on_mouse_down(tree, lx, ly, MouseButton::Left);
                                        let (local_x, local_y) = router.last_hit_local();
                                        let (bounds_x, bounds_y) = router.last_hit_bounds_pos();
                                        let (bounds_width, bounds_height) = router.last_hit_bounds();
                                        for event in pending_events.iter_mut() {
                                            event.mouse_x = lx;
                                            event.mouse_y = ly;
                                            event.local_x = local_x;
                                            event.local_y = local_y;
                                            event.bounds_x = bounds_x;
                                            event.bounds_y = bounds_y;
                                            event.bounds_width = bounds_width;
                                            event.bounds_height = bounds_height;
                                        }
                                    }
                                    TouchEvent::Moved { x, y, .. } => {
                                        let lx = x / scale;
                                        let ly = y / scale;

                                        // Use occlusion-aware hit testing for touch move as well
                                        let overlay_bounds = windowed_ctx.overlay_manager.get_visible_overlay_bounds();
                                        let overlay_layer_id = tree.query_by_id(
                                            blinc_layout::widgets::overlay::OVERLAY_LAYER_ID
                                        );
                                        router.on_mouse_move_with_occlusion(
                                            tree,
                                            lx,
                                            ly,
                                            &overlay_bounds,
                                            overlay_layer_id,
                                        );

                                        for event in pending_events.iter_mut() {
                                            event.mouse_x = lx;
                                            event.mouse_y = ly;
                                        }
                                    }
                                    TouchEvent::Ended { x, y, .. } => {
                                        let lx = x / scale;
                                        let ly = y / scale;
                                        router.on_mouse_up(tree, lx, ly, MouseButton::Left);
                                        for event in pending_events.iter_mut() {
                                            event.mouse_x = lx;
                                            event.mouse_y = ly;
                                        }
                                    }
                                    TouchEvent::Cancelled { .. } => {
                                        // Touch cancelled - treat like mouse leave
                                        // This will emit POINTER_UP if there was a pressed target
                                        router.on_mouse_leave();
                                    }
                                },
                                InputEvent::Scroll { delta_x, delta_y, phase } => {
                                    let (mx, my) = router.mouse_position();
                                    // Scroll deltas are also in physical pixels, convert to logical
                                    let ldx = delta_x;
                                    let ldy = delta_y;

                                    tracing::trace!(
                                        "InputEvent::Scroll received: pos=({:.1}, {:.1}) delta=({:.1}, {:.1}) phase={:?}",
                                        mx, my, ldx, ldy, phase
                                    );

                                    // Check if gesture ended (finger lifted from trackpad)
                                    // This happens before momentum ends
                                    if phase == blinc_platform::ScrollPhase::Ended {
                                        gesture_ended = true;
                                    }

                                    // Use nested scroll support - get hit result for smart dispatch
                                    // Store mouse position and delta for dispatch phase
                                    // We'll re-do hit test in dispatch phase since we need mutable borrow
                                    scroll_info = Some((mx, my, ldx, ldy));
                                }
                                InputEvent::Pinch { scale } => {
                                    let (mx, my) = router.mouse_position();
                                    pinch_info = Some((mx, my, scale));
                                }
                                InputEvent::ScrollEnd => {
                                    // Scroll momentum ended - full stop
                                    scroll_ended = true;
                                }
                            }

                            router.clear_event_callback();
                            (pending_events, keyboard_events, scroll_ended, gesture_ended, scroll_info, pinch_info)
                        } else {
                            (Vec::new(), Vec::new(), false, false, None, None)
                        };

                        // Second phase: dispatch events with mutable borrow
                        // This automatically marks the tree dirty when handlers fire
                        if let Some(ref mut tree) = render_tree {
                            // IMPORTANT: Process gesture_ended BEFORE scroll delta dispatch
                            // When gesture ends while overscrolling, we start bounce which
                            // sets state to Bouncing. Then apply_scroll_delta will early-return
                            // and ignore the momentum delta that came with this same event.
                            if gesture_ended {
                                tree.on_gesture_end();
                                // Request redraw to animate bounce-back
                                window.request_redraw();
                            }

                            // Handle scroll with nested scroll support
                            // Skip scroll delta entirely if gesture just ended - the delta
                            // from the same event as gesture_ended is the last finger movement,
                            // not momentum, but we still want to ignore it for instant snap-back
                            //
                            // Also skip scroll when an overlay with an actual backdrop is open to prevent
                            // background content from scrolling while dropdown/modal is visible.
                            // Note: We only check has_blocking_overlay(), not has_dismissable_overlay(),
                            // because overlays with dismiss_on_click_outside (like popovers) should allow
                            // scroll events to pass through to content behind them.
                            let has_overlay_backdrop = ctx
                                .as_ref()
                                .map(|c| c.overlay_manager.has_blocking_overlay())
                                .unwrap_or(false);

                            if let Some((mouse_x, mouse_y, scale)) = pinch_info {
                                if has_overlay_backdrop {
                                    tracing::trace!("Skipping pinch - overlay with backdrop is visible");
                                } else if let Some(ref mut windowed_ctx) = ctx {
                                    let router = &mut windowed_ctx.event_router;
                                    if let Some(hit) = router.hit_test(tree, mouse_x, mouse_y) {
                                        tree.dispatch_pinch_chain(&hit, mouse_x, mouse_y, scale);
                                    }
                                }
                            }

                            if let Some((mouse_x, mouse_y, delta_x, delta_y)) = scroll_info {
                                // Skip if gesture ended in this same event - go straight to bounce
                                if gesture_ended {
                                    tracing::trace!("Skipping scroll delta - gesture ended, bouncing");
                                } else if has_overlay_backdrop {
                                    // Skip scroll when overlay is visible to prevent background scrolling
                                    tracing::trace!("Skipping scroll delta - overlay with backdrop is visible");
                                } else {
                                    tracing::trace!(
                                        "Scroll dispatch: pos=({:.1}, {:.1}) delta=({:.1}, {:.1})",
                                        mouse_x, mouse_y, delta_x, delta_y
                                    );

                                    // Update overlay positions for overlays with follows_scroll enabled
                                    // Use the singleton overlay manager since components use get_overlay_manager()
                                    if OverlayContext::is_initialized() {
                                        let mgr = get_overlay_manager();
                                        if mgr.handle_scroll(delta_y) {
                                            // Apply scroll offsets to render tree for visual movement
                                            for (element_id, offset_y) in mgr.get_scroll_offsets() {
                                                if let Some(node_id) = tree.query_by_id(&element_id) {
                                                    tree.set_scroll_offset(node_id, 0.0, offset_y);
                                                }
                                            }
                                            window.request_redraw();
                                        }
                                    }

                                    // Re-do hit test with mutable borrow to get ancestor chain
                                    // Then use dispatch_scroll_chain for proper nested scroll handling
                                    if let Some(ref mut windowed_ctx) = ctx {
                                        let router = &mut windowed_ctx.event_router;
                                        if let Some(hit) = router.hit_test(tree, mouse_x, mouse_y) {
                                            tracing::trace!(
                                                "Hit: node={:?}, ancestors={:?}",
                                                hit.node, hit.ancestors
                                            );
                                            tree.dispatch_scroll_chain(
                                                hit.node,
                                                &hit.ancestors,
                                                mouse_x,
                                                mouse_y,
                                                delta_x,
                                                delta_y,
                                            );
                                        }
                                    }
                                }
                            }

                            // Dispatch mouse/touch events (scroll is handled above with nested support)
                            if let Some(ref mut windowed_ctx) = ctx {
                                let router = &windowed_ctx.event_router;
                                for event in pending_events {
                                    // Skip scroll events - already handled with nested scroll support
                                    if event.event_type == blinc_core::events::event_types::SCROLL {
                                        continue;
                                    }
                                    // Look up the correct bounds for this specific node.
                                    // When events bubble from a child to a parent handler,
                                    // we need the parent's bounds, not the original hit target's bounds.
                                    let (bounds_x, bounds_y, bounds_width, bounds_height) =
                                        router.get_node_bounds(event.node_id).unwrap_or((
                                            event.bounds_x,
                                            event.bounds_y,
                                            event.bounds_width,
                                            event.bounds_height,
                                        ));
                                    let local_x = event.mouse_x - bounds_x;
                                    let local_y = event.mouse_y - bounds_y;
                                    tree.dispatch_event_full(
                                        event.node_id,
                                        event.event_type,
                                        event.mouse_x,
                                        event.mouse_y,
                                        local_x,
                                        local_y,
                                        bounds_x,
                                        bounds_y,
                                        bounds_width,
                                        bounds_height,
                                        event.drag_delta_x,
                                        event.drag_delta_y,
                                        1.0,
                                    );
                                }
                            }

                            // Note: Overlay events are now dispatched through the main tree
                            // since overlays are composed into the main tree via build_overlay_layer()

                            // Dispatch keyboard events
                            // Use broadcast instead of bubbling to handle focus correctly after tree rebuilds.
                            // Text inputs track their own focus state internally via `s.visual.is_focused()`,
                            // so broadcasting to all handlers is safe - only the focused one will process.
                            for event in keyboard_events {
                                if event.event_type == blinc_core::events::event_types::TEXT_INPUT {
                                    if let Some(c) = event.key_char {
                                        // Broadcast to all text input handlers
                                        // Each handler checks its own focus state internally
                                        tree.broadcast_text_input_event(
                                            c,
                                            event.shift,
                                            event.ctrl,
                                            event.alt,
                                            event.meta,
                                        );
                                    }
                                } else {
                                    // Broadcast KEY_DOWN to all key handlers
                                    tree.broadcast_key_event(
                                        event.event_type,
                                        event.key_code,
                                        event.shift,
                                        event.ctrl,
                                        event.alt,
                                        event.meta,
                                    );
                                }
                            }

                            // If scroll momentum ended, notify scroll physics
                            if scroll_ended {
                                tree.on_scroll_end();
                                // Request redraw to animate bounce-back
                                window.request_redraw();
                            }
                        }
                    }

                    Event::Frame => {
                        if let (
                            Some(ref mut blinc_app),
                            Some(ref surf),
                            Some(ref config),
                            Some(ref mut windowed_ctx),
                            Some(ref mut rs),
                        ) = (&mut app, &surface, &surface_config, &mut ctx, &mut render_state)
                        {
                            // Get current frame
                            let frame = match surf.get_current_texture() {
                                Ok(f) => f,
                                Err(wgpu::SurfaceError::Lost) => {
                                    surf.configure(blinc_app.device(), config);
                                    return ControlFlow::Continue;
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    tracing::error!("Out of GPU memory");
                                    #[cfg(feature = "webview")]
                                    webview_lifecycle.dispose();
                                    return ControlFlow::Exit;
                                }
                                Err(e) => {
                                    tracing::warn!("Surface error: {:?}", e);
                                    return ControlFlow::Continue;
                                }
                            };

                            let view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());

                            // Update context from window
                            windowed_ctx.update_from_window(window);

                            // Update viewport for lazy loading visibility checks
                            // Uses logical pixels (width/height) as that's what layout uses
                            rs.set_viewport_size(windowed_ctx.width, windowed_ctx.height);

                            // Get current time for animation updates (used in multiple phases)
                            let current_time = elapsed_ms();

                            // Clear overlays from previous frame (cursor, selection, focus ring)
                            // These are re-added during rendering if still active
                            rs.clear_overlays();

                            // Tick scroll physics and sync ScrollRef state BEFORE any rebuilds
                            // This ensures ScrollRef has up-to-date values when stateful components
                            // query scroll position during rebuild
                            let scroll_animating = if let Some(ref mut tree) = render_tree {
                                let animating = tree.tick_scroll_physics(current_time);
                                tree.process_pending_scroll_refs();
                                animating
                            } else {
                                false
                            };

                            // =========================================================
                            // PHASE 1: Check if tree structure needs rebuild
                            // Only structural changes require tree rebuild
                            // =========================================================

                            // Check if event handlers marked anything dirty (auto-rebuild)
                            if let Some(ref tree) = render_tree {
                                if tree.needs_rebuild() {
                                    tracing::debug!("Rebuild triggered by: dirty_tracker");
                                    needs_rebuild = true;
                                }
                            }

                            // Check if element refs were modified (triggers rebuild)
                            if ref_dirty_flag.swap(false, Ordering::SeqCst) {
                                tracing::debug!("Rebuild triggered by: ref_dirty_flag (State::set)");
                                needs_rebuild = true;
                            }

                            // Check if text widgets requested a rebuild (focus/text changes)
                            if blinc_layout::widgets::take_needs_rebuild() {
                                tracing::debug!("Rebuild triggered by: text widget state change");
                                needs_rebuild = true;
                            }

                            // Check if a full relayout was requested (e.g., theme changes)
                            if blinc_layout::widgets::take_needs_relayout() {
                                tracing::debug!("Relayout triggered by: theme or global state change");
                                needs_relayout = true;
                            }

                            // Process pending motion exit starts BEFORE overlay update
                            // This is critical: when an overlay closes, it queues a motion exit via
                            // query_motion(key).exit(). The overlay's update() method then checks
                            // if the motion is done animating. If we don't process the exit queue
                            // first, the motion won't be in Exiting state yet, and update() will
                            // incorrectly think the exit animation is complete.
                            rs.process_global_motion_exit_starts();
                            rs.process_global_motion_exit_cancels();
                            // Process suspended motion starts queued via query_motion(key).start()
                            rs.process_global_motion_starts();

                            // Sync motion states to shared store so overlay can query them
                            // This must happen after processing exits but before overlay update
                            rs.sync_shared_motion_states();

                            // Update overlay manager viewport and state for subtree rebuilds
                            // This must happen BEFORE checking is_dirty() so build_overlay_layer() works correctly
                            windowed_ctx.overlay_manager.set_viewport_with_scale(
                                windowed_ctx.width,
                                windowed_ctx.height,
                                windowed_ctx.scale_factor as f32,
                            );
                            windowed_ctx.overlay_manager.update(current_time);

                            // Check if overlay content changed (new overlay opened/closed)
                            // NOTE: We only rebuild on actual content changes, NOT during animations.
                            // Animation visual updates (backdrop opacity, motion transforms) are handled
                            // by the motion system and render-time interpolation, not content rebuilds.
                            // Rebuilding during animation breaks event handlers because node IDs change.
                            let overlay_content_dirty = windowed_ctx.overlay_manager.is_dirty();

                            if overlay_content_dirty {
                                tracing::debug!(
                                    "Overlay rebuild: dirty={}, has_visible={}",
                                    overlay_content_dirty,
                                    windowed_ctx.overlay_manager.has_visible_overlays()
                                );
                                // Look up the overlay layer node by its element ID
                                if let Some(overlay_node_id) = element_registry.get(
                                    blinc_layout::widgets::overlay::OVERLAY_LAYER_ID
                                ) {
                                    tracing::debug!("Overlay changed - queueing subtree rebuild for node {:?}", overlay_node_id);
                                    // Build the new overlay content and queue the subtree rebuild
                                    let overlay_content = windowed_ctx.overlay_manager.build_overlay_layer();
                                    blinc_layout::queue_subtree_rebuild(overlay_node_id, overlay_content);
                                } else {
                                    tracing::warn!("Overlay changed but node '{}' not found in registry - will rebuild on next frame",
                                        blinc_layout::widgets::overlay::OVERLAY_LAYER_ID);
                                }
                                // Consume the dirty flag
                                windowed_ctx.overlay_manager.take_dirty();
                            }

                            // Check if stateful elements requested a redraw (hover/press changes)
                            // Apply incremental prop updates without full rebuild
                            let has_stateful_updates = blinc_layout::take_needs_redraw();
                            let has_pending_rebuilds = blinc_layout::has_pending_subtree_rebuilds();

                            if has_stateful_updates || has_pending_rebuilds {
                                if has_stateful_updates {
                                    tracing::debug!("Redraw requested by: stateful state change");
                                }

                                // Get all pending prop updates
                                let prop_updates = blinc_layout::take_pending_prop_updates();
                                let had_prop_updates = !prop_updates.is_empty();

                                // Apply prop updates to the main tree
                                // (Overlays are now part of the main tree, so all nodes are here)
                                if let Some(ref mut tree) = render_tree {
                                    for (node_id, props) in &prop_updates {
                                        tree.update_render_props(*node_id, |p| *p = props.clone());
                                    }
                                }

                                // Process subtree rebuilds (from stateful changes OR overlay changes)
                                let mut needs_layout = false;
                                if let Some(ref mut tree) = render_tree {
                                    needs_layout = tree.process_pending_subtree_rebuilds();
                                }

                                if needs_layout {
                                    if let Some(ref mut tree) = render_tree {
                                        tracing::debug!("Subtree rebuilds processed, recomputing layout");
                                        tree.apply_stylesheet_layout_overrides();
                                        tree.compute_layout(windowed_ctx.width, windowed_ctx.height);
                                        // Begin/end motion frame to track which motions are still in tree
                                        rs.begin_stable_motion_frame();
                                        tree.initialize_motion_animations(rs);
                                        rs.end_stable_motion_frame();
                                        rs.process_global_motion_replays();
                                        // Start CSS animations for elements with animation properties
                                        tree.start_all_css_animations();
                                    }
                                }
                                if had_prop_updates && !needs_layout {
                                    tracing::trace!("Visual-only prop updates, skipping layout");
                                }

                                // Request window redraw without rebuild
                                window.request_redraw();
                            }

                            // =========================================================
                            // PHASE 2: Build/rebuild tree only for structural changes
                            // This must happen BEFORE tick() so motion animations are available
                            // =========================================================

                            // Begin stable motion frame tracking
                            // This clears the "used" set so we can detect which motions are no longer in the tree
                            rs.begin_stable_motion_frame();

                            if needs_rebuild || render_tree.is_none() {
                                // Reset call counters for stable key generation
                                reset_call_counters();

                                // Reset stable motions so they replay on full rebuild
                                // This ensures motion animations play when UI is reconstructed
                                rs.reset_stable_motions_for_rebuild();

                                // Note: Viewport and overlay state are already updated in PHASE 1
                                // so build_overlay_layer() has correct dimensions

                                // Build UI element tree
                                let user_ui = ui_builder(windowed_ctx);

                                // Compose user UI with overlay layer using a regular Div container
                                // We use position:relative with the overlay absolutely positioned on top.
                                let overlay_layer = windowed_ctx.overlay_manager.build_overlay_layer();
                                let ui = div()
                                    .w(windowed_ctx.width)
                                    .h(windowed_ctx.height)
                                    .relative() // positioning context for overlay
                                    .child(user_ui)
                                    .child(overlay_layer);

                                // Use incremental update if we have an existing tree
                                // BUT: Skip incremental update during resize - do full rebuild instead
                                // This ensures parent constraints properly propagate to all children
                                if let Some(ref mut existing_tree) = render_tree {
                                    if needs_relayout {
                                        // Window resize: bypass incremental update, do full rebuild
                                        // This ensures proper constraint propagation from parents to children
                                        tracing::debug!("Window resize: full tree rebuild (bypassing incremental update)");

                                        // Clear layout bounds storages before rebuild
                                        existing_tree.clear_layout_bounds_storages();

                                        // Full rebuild: create new tree from element with shared registry
                                        // Pass registry to from_element_with_registry so IDs are registered during build
                                        let mut tree = RenderTree::from_element_with_registry(
                                            &ui,
                                            Arc::clone(&element_registry),
                                        );

                                        // Set animation scheduler for scroll bounce springs
                                        tree.set_animations(&windowed_ctx.animations);

                                        // Share the CSS animation store (ticked by scheduler thread)
                                        tree.set_css_anim_store(Arc::clone(&css_anim_store));

                                        // Set DPI scale factor for HiDPI rendering
                                        tree.set_scale_factor(windowed_ctx.scale_factor as f32);

                                        // Set CSS stylesheet for automatic style application
                                        if let Some(ref stylesheet) = windowed_ctx.stylesheet {
                                            tree.set_stylesheet_arc(stylesheet.clone());
                                        }
                                        // Apply base CSS styles to all registered elements
                                        // (stylesheet was set after tree construction, so collect_render_props missed them)
                                        tree.apply_stylesheet_base_styles();
                                        // Apply stylesheet layout overrides before layout computation
                                        tree.apply_stylesheet_layout_overrides();

                                        // Compute layout with new viewport dimensions
                                        tree.compute_layout(windowed_ctx.width, windowed_ctx.height);

                                        // Initialize motion animations for any nodes wrapped in motion() containers
                                        tree.initialize_motion_animations(rs);
                                        // End motion frame to detect unmounted motions and trigger exit animations
                                        rs.end_stable_motion_frame();
                                        // Process any motion replay requests queued during tree building
                                        rs.process_global_motion_replays();
                                        // Start CSS animations for elements with animation properties
                                        tree.start_all_css_animations();

                                        // Replace existing tree with fresh one
                                        *existing_tree = tree;

                                        // Clear relayout flag after full rebuild
                                        needs_relayout = false;
                                    } else {
                                        // Normal incremental update (no resize)
                                        use blinc_layout::UpdateResult;

                                        // Update stylesheet in case it changed between frames
                                        if let Some(ref stylesheet) = windowed_ctx.stylesheet {
                                            existing_tree.set_stylesheet_arc(stylesheet.clone());
                                        }

                                        let update_result = existing_tree.incremental_update(&ui);

                                        match update_result {
                                            UpdateResult::NoChanges => {
                                                tracing::debug!("Incremental update: NoChanges - skipping rebuild");
                                            }
                                            UpdateResult::VisualOnly => {
                                                tracing::debug!("Incremental update: VisualOnly - skipping layout");
                                                // Props already updated in-place by incremental_update
                                            }
                                            UpdateResult::LayoutChanged => {
                                                // Layout changed - recompute layout
                                                tracing::debug!("Incremental update: LayoutChanged - recomputing layout");
                                                existing_tree.apply_stylesheet_layout_overrides();
                                                existing_tree.compute_layout(windowed_ctx.width, windowed_ctx.height);
                                            }
                                            UpdateResult::ChildrenChanged => {
                                                // Children changed - subtrees were rebuilt in place
                                                tracing::debug!("Incremental update: ChildrenChanged - subtrees rebuilt");

                                                // Recompute layout since structure changed
                                                existing_tree.apply_stylesheet_layout_overrides();
                                                existing_tree.compute_layout(windowed_ctx.width, windowed_ctx.height);

                                                // Initialize motion animations for any new nodes wrapped in motion() containers
                                                existing_tree.initialize_motion_animations(rs);
                                                // End motion frame to detect unmounted motions and trigger exit animations
                                                rs.end_stable_motion_frame();

                                                // Process any global motion replays that were queued during tree building
                                                rs.process_global_motion_replays();
                                                // Start CSS animations for elements with animation properties
                                                existing_tree.start_all_css_animations();
                                            }
                                        }
                                    }
                                } else {
                                    // No existing tree - create new with shared registry
                                    let mut tree = RenderTree::from_element_with_registry(
                                        &ui,
                                        Arc::clone(&element_registry),
                                    );

                                    // Set animation scheduler for scroll bounce springs
                                    tree.set_animations(&windowed_ctx.animations);

                                    // Share the CSS animation store (ticked by scheduler thread)
                                    tree.set_css_anim_store(Arc::clone(&css_anim_store));

                                    // Set DPI scale factor for HiDPI rendering
                                    tree.set_scale_factor(windowed_ctx.scale_factor as f32);

                                    // Set CSS stylesheet for automatic style application
                                    if let Some(ref stylesheet) = windowed_ctx.stylesheet {
                                        tree.set_stylesheet_arc(stylesheet.clone());
                                    }
                                    // Apply base CSS styles to all registered elements
                                    // (stylesheet was set after tree construction, so collect_render_props missed them)
                                    tree.apply_stylesheet_base_styles();
                                    // Apply stylesheet layout overrides before layout computation
                                    tree.apply_stylesheet_layout_overrides();

                                    // Compute layout in logical pixels
                                    tree.compute_layout(windowed_ctx.width, windowed_ctx.height);

                                    // Initialize motion animations for any nodes wrapped in motion() containers
                                    tree.initialize_motion_animations(rs);
                                    // End motion frame to detect unmounted motions and trigger exit animations
                                    rs.end_stable_motion_frame();

                                    // Process any global motion replays that were queued during tree building
                                    rs.process_global_motion_replays();
                                    // Start CSS animations for elements with animation properties
                                    tree.start_all_css_animations();

                                    render_tree = Some(tree);
                                }

                                needs_rebuild = false;
                                let was_first_rebuild = windowed_ctx.rebuild_count == 0;
                                windowed_ctx.rebuild_count = windowed_ctx.rebuild_count.saturating_add(1);

                                // Execute on_ready callbacks after first rebuild
                                if was_first_rebuild {
                                    if let Ok(mut callbacks) = ready_callbacks.lock() {
                                        for callback in callbacks.drain(..) {
                                            callback();
                                        }
                                    }
                                }
                            } else {
                                // No rebuild needed - still need to end the motion frame
                                // If an existing tree exists, initialize motions to mark them as used
                                if let Some(ref tree) = render_tree {
                                    tree.initialize_motion_animations(rs);
                                }
                                rs.end_stable_motion_frame();
                            }

                            // Note: on_ready callbacks are only executed after the FIRST rebuild
                            // (in the was_first_rebuild block above). Callbacks registered
                            // after the first rebuild are executed immediately since the UI
                            // is already ready at that point.

                            // =========================================================
                            // Optional internal e2e scripts (deterministic input simulation)
                            // =========================================================
                            if !e2e_script_ran {
                                if let Some(script) = e2e_script {
                                    if let Some(ref mut tree) = render_tree {
                                        if windowed_ctx.rebuild_count > 0 {
                                            // Only run once per process after the first build is complete.
                                            e2e_script_ran = true;
                                            match script {
                                                E2eScript::GallerySidebarClickAfterScroll => {
                                                    let router = &mut windowed_ctx.event_router;
                                                    let window_w = windowed_ctx.width;
                                                    let window_h = windowed_ctx.height;

                                                    let selected_before: Option<usize> =
                                                        read_keyed_state(&hooks, &reactive, "charts_gallery_selected");
                                                    tracing::info!(
                                                        "e2e_script: gallery_sidebar_click_after_scroll selected_before={:?}",
                                                        selected_before
                                                    );

                                                    let x_search_max = 360.0;
                                                    let Some((probe_x, probe_y, probe_id)) =
                                                        e2e_find_hit_point_with_id_prefix(
                                                            tree,
                                                            router,
                                                            window_w,
                                                            window_h,
                                                            x_search_max,
                                                            "charts_gallery_sidebar_item_",
                                                        )
                                                    else {
                                                        eprintln!(
                                                            "e2e script error: could not find sidebar hit point (prefix='charts_gallery_sidebar_item_')"
                                                        );
                                                        std::process::exit(1);
                                                    };
                                                    tracing::info!(
                                                        "e2e_script: probe=({:.1}, {:.1}) hit_id={}",
                                                        probe_x,
                                                        probe_y,
                                                        probe_id
                                                    );

                                                    let Some(sidebar_scroll_node) =
                                                        tree.query_by_id("charts_gallery_sidebar_scroll")
                                                    else {
                                                        eprintln!(
                                                            "e2e script error: missing element id 'charts_gallery_sidebar_scroll'"
                                                        );
                                                        std::process::exit(1);
                                                    };

                                                    let target_index: usize = std::env::var(
                                                        "BLINC_E2E_GALLERY_TARGET_INDEX",
                                                    )
                                                    .ok()
                                                    .and_then(|v| v.trim().parse::<usize>().ok())
                                                    .unwrap_or(8);
                                                    let target_id = format!(
                                                        "charts_gallery_sidebar_item_{}",
                                                        target_index
                                                    );

                                                    let mut click_point: Option<(f32, f32)> = None;
                                                    for _ in 0..180 {
                                                        if let Some((x, y)) = e2e_find_hit_point_with_exact_id(
                                                            tree,
                                                            router,
                                                            window_w,
                                                            window_h,
                                                            x_search_max,
                                                            &target_id,
                                                        ) {
                                                            click_point = Some((x, y));
                                                            break;
                                                        }
                                                        // Scroll down (content moves up).
                                                        tree.dispatch_scroll_event(
                                                            sidebar_scroll_node,
                                                            probe_x,
                                                            probe_y,
                                                            0.0,
                                                            -72.0,
                                                        );
                                                    }

                                                    let Some((click_x, click_y)) = click_point else {
                                                        eprintln!(
                                                            "e2e script error: failed to scroll to target id '{target_id}'"
                                                        );
                                                        std::process::exit(1);
                                                    };

                                                    // Simulate click at a point that actually hits the target.
                                                    tracing::info!(
                                                        "e2e_script: target_id={} click=({:.1}, {:.1})",
                                                        target_id,
                                                        click_x,
                                                        click_y
                                                    );
                                                    let mut down_events = router.on_mouse_down(
                                                        tree,
                                                        click_x,
                                                        click_y,
                                                        MouseButton::Left,
                                                    );
                                                    down_events.extend(router.on_mouse_up(
                                                        tree,
                                                        click_x,
                                                        click_y,
                                                        MouseButton::Left,
                                                    ));
                                                    for (node, event_type) in down_events {
                                                        let (bx, by, bw, bh) = router
                                                            .get_node_bounds(node)
                                                            .unwrap_or((0.0, 0.0, 0.0, 0.0));
                                                        tree.dispatch_event_full(
                                                            node,
                                                            event_type,
                                                            click_x,
                                                            click_y,
                                                            click_x - bx,
                                                            click_y - by,
                                                            bx,
                                                            by,
                                                            bw,
                                                            bh,
                                                            0.0,
                                                            0.0,
                                                            1.0,
                                                        );
                                                    }

                                                    let selected_after: Option<usize> =
                                                        read_keyed_state(&hooks, &reactive, "charts_gallery_selected");
                                                    tracing::info!(
                                                        "e2e_script: gallery_sidebar_click_after_scroll selected_after={:?} expected={}",
                                                        selected_after,
                                                        target_index
                                                    );
                                                    if selected_after != Some(target_index) {
                                                        eprintln!(
                                                            "e2e script error: click did not update selection (got {:?}, expected {})",
                                                            selected_after,
                                                            target_index
                                                        );
                                                        std::process::exit(1);
                                                    }

                                                    println!(
                                                        "e2e script ok: gallery_sidebar_click_after_scroll target_index={}",
                                                        target_index
                                                    );

                                                    if e2e_script_exit {
                                                        std::process::exit(0);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // =========================================================
                            // PHASE 3: Tick animations and dynamic render state
                            // This must happen AFTER tree rebuild so motions are initialized
                            // =========================================================

                            // Process any pending motion exit cancellations
                            // This must happen before tick() so cancelled motions don't continue exiting
                            rs.process_global_motion_exit_cancels();

                            // Process any pending motion exit starts (explicit exit triggers)
                            rs.process_global_motion_exit_starts();

                            // Process suspended motion starts queued via query_motion(key).start()
                            rs.process_global_motion_starts();

                            // Tick render state (handles cursor blink, color animations, etc.)
                            // This updates dynamic properties without touching tree structure
                            let _animations_active = rs.tick(current_time);

                            // Tick CSS animations/transitions synchronously on the main thread.
                            // The scheduler's bg thread drives 120fps redraws via wake_callback,
                            // but actual ticking is done here to stay in phase with rendering.
                            let dt_ms = if last_frame_time_ms > 0 {
                                (current_time - last_frame_time_ms) as f32
                            } else {
                                16.0
                            };
                            let css_active = if let Some(ref tree) = render_tree {
                                let store = tree.css_anim_store();
                                let mut s = store.lock().unwrap();
                                let (anim, trans) = s.tick(dt_ms);
                                drop(s);
                                anim || trans || tree.css_has_active()
                            } else {
                                false
                            };
                            last_frame_time_ms = current_time;

                            // Sync motion states to shared store for query_motion API
                            rs.sync_shared_motion_states();

                            // Tick theme animation (handles color interpolation during theme transitions)
                            let theme_animating = blinc_theme::ThemeState::get().tick();

                            // Note: scroll physics tick moved to before PHASE 1 (before any rebuilds)
                            // so that ScrollRef has up-to-date values when stateful components rebuild

                            // =========================================================
                            // PHASE 4: Render
                            // Combines stable tree structure with dynamic render state
                            // =========================================================

                            // Apply CSS state styles (:hover, :active, :focus) from stylesheet
                            // This also detects property changes and starts new transitions
                            if let Some(ref mut tree) = render_tree {
                                if tree.stylesheet().is_some() {
                                    tree.apply_stylesheet_state_styles(&windowed_ctx.event_router);
                                }
                            }

                            // Apply CSS animation/transition values AFTER state styles
                            // (state styles reset to base, animations must override)
                            if css_active || !render_tree.as_ref().map_or(true, |t| t.css_transitions_empty()) {
                                if let Some(ref mut tree) = render_tree {
                                    tree.apply_all_css_animation_props();
                                    tree.apply_all_css_transition_props();
                                    if tree.apply_animated_layout_props() {
                                        tree.compute_layout(windowed_ctx.width, windowed_ctx.height);
                                    }
                                }
                            }

                            if let Some(ref tree) = render_tree {
                                // Render with motion animations
                                // Use physical pixel dimensions for the render surface
                                let result = blinc_app.render_tree_with_motion(
                                    tree,
                                    rs,
                                    &view,
                                    windowed_ctx.physical_width as u32,
                                    windowed_ctx.physical_height as u32,
                                );
                                if let Err(e) = result {
                                    tracing::error!("Render error: {}", e);
                                }
                            }

                            // Optional: capture + validate rendered pixels for e2e.
                            //
                            // This avoids relying on OS-level screenshots (which can be black in CI
                            // or require Screen Recording permission). Captures are read back from
                            // the swapchain frame via COPY_SRC.
                            if e2e_enabled && e2e_captures_done < e2e_max_captures {
                                let mut should_capture = false;
                                let mut consumed_trigger = false;

                                if e2e_capture_on_start && e2e_captures_done == 0 {
                                    should_capture = true;
                                } else if let Some(ref trigger) = e2e_trigger_path {
                                    if trigger.exists() {
                                        should_capture = true;
                                        consumed_trigger = true;
                                    }
                                }

                                if should_capture && render_tree.is_some() {
                                    let capture_index = e2e_captures_done + 1; // 1-based

                                    let width = windowed_ctx.physical_width as u32;
                                    let height = windowed_ctx.physical_height as u32;
                                    let bytes_per_row = padded_bytes_per_row(width);
                                    let buffer_size = (bytes_per_row as u64) * (height as u64);

                                    let buffer = blinc_app.device().create_buffer(&wgpu::BufferDescriptor {
                                        label: Some("blinc_e2e_readback"),
                                        size: buffer_size,
                                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                                        mapped_at_creation: false,
                                    });

                                    let mut encoder =
                                        blinc_app.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                            label: Some("blinc_e2e_copy_encoder"),
                                        });

                                    encoder.copy_texture_to_buffer(
                                        wgpu::ImageCopyTexture {
                                            texture: &frame.texture,
                                            mip_level: 0,
                                            origin: wgpu::Origin3d::ZERO,
                                            aspect: wgpu::TextureAspect::All,
                                        },
                                        wgpu::ImageCopyBuffer {
                                            buffer: &buffer,
                                            layout: wgpu::ImageDataLayout {
                                                offset: 0,
                                                bytes_per_row: Some(bytes_per_row),
                                                rows_per_image: Some(height),
                                            },
                                        },
                                        wgpu::Extent3d {
                                            width,
                                            height,
                                            depth_or_array_layers: 1,
                                        },
                                    );

                                    blinc_app.queue().submit(std::iter::once(encoder.finish()));

                                    let buffer_slice = buffer.slice(..);
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                                        tx.send(result).ok();
                                    });
                                    blinc_app.device().poll(wgpu::Maintain::Wait);
                                    if let Err(e) = rx
                                        .recv()
                                        .unwrap_or(Err(wgpu::BufferAsyncError))
                                    {
                                        eprintln!("e2e error: failed to map readback buffer: {e}");
                                        std::process::exit(1);
                                    }

                                    let data = buffer_slice.get_mapped_range();
                                    let Some(rgba) =
                                        bgra_or_rgba_to_rgba(config.format, &data, width, height)
                                    else {
                                        eprintln!(
                                            "e2e error: unsupported surface format for readback: {:?}",
                                            config.format
                                        );
                                        std::process::exit(1);
                                    };
                                    drop(data);
                                    buffer.unmap();

                                    let out_path = e2e_capture_path
                                        .as_ref()
                                        .map(|base| e2e_output_path(base, capture_index));
                                    if let Some(path) = out_path.as_ref() {
                                        let parent: Option<&std::path::Path> = if path.is_dir() {
                                            Some(path.as_path())
                                        } else {
                                            path.parent()
                                        };
                                        if let Some(parent) = parent {
                                            let _ = std::fs::create_dir_all(parent);
                                        }
                                        if let Err(e) =
                                            e2e_save_png_minimal_rgba(&rgba, width, height, path)
                                        {
                                            eprintln!(
                                                "e2e error: failed to write png {}: {e}",
                                                path.display()
                                            );
                                            std::process::exit(1);
                                        }
                                    }

                                    let (blue, warm, total) = e2e_count_pixels(&rgba, width, height);

                                    if let Some(expect) = e2e_expect {
                                        let threshold = e2e_threshold(expect, width, height, total);
                                        match expect {
                                            E2eExpect::Blueish => {
                                                if blue < threshold {
                                                    eprintln!(
                                                        "e2e error: expected colored line pixels, got blue={blue} (threshold={threshold}, total={total})"
                                                    );
                                                    if let Some(path) = out_path.as_ref() {
                                                        eprintln!("e2e png: {}", path.display());
                                                    }
                                                    std::process::exit(1);
                                                }
                                            }
                                            E2eExpect::Warm => {
                                                if warm < threshold {
                                                    eprintln!(
                                                        "e2e error: expected warm heatmap pixels, got warm={warm} (threshold={threshold}, total={total})"
                                                    );
                                                    if let Some(path) = out_path.as_ref() {
                                                        eprintln!("e2e png: {}", path.display());
                                                    }
                                                    std::process::exit(1);
                                                }
                                            }
                                        }
                                    }

                                    if consumed_trigger {
                                        if let Some(ref trigger) = e2e_trigger_path {
                                            let _ = std::fs::remove_file(trigger);
                                        }
                                    }

                                    e2e_captures_done += 1;
                                    println!(
                                        "e2e ok: capture={} {}x{} blue={} warm={} total={}",
                                        capture_index, width, height, blue, warm, total
                                    );
                                    if let Some(path) = out_path.as_ref() {
                                        println!("e2e png: {}", path.display());
                                    }

                                    if e2e_exit && e2e_captures_done >= e2e_max_captures {
                                        std::process::exit(0);
                                    }
                                }
                            }

                            // =========================================================
                            // PHASE 4b: Overlay state management (overlays now in main tree)
                            // Overlays are composed into the main tree via build_overlay_layer()
                            // so they share the same event routing and incremental update path.
                            // =========================================================

                            // Clear dirty flags for overlays (they've been processed in tree build)
                            let _content_dirty = windowed_ctx.overlay_manager.take_dirty();
                            let _animation_dirty = windowed_ctx.overlay_manager.take_animation_dirty();

                            // Track overlay visibility for triggering rebuilds
                            let has_visible_overlays = windowed_ctx.overlay_manager.has_visible_overlays();
                            windowed_ctx.had_visible_overlays = has_visible_overlays;

                            frame.present();

                            // =========================================================
                            // PHASE 5: Request next frame if animations are active
                            // This ensures smooth animation without waiting for events
                            // =========================================================

                            // Check if background animation thread signaled that redraw is needed
                            // The background thread runs at 120fps and sets this flag when
                            // there are active animations (springs, keyframes, timelines)
                            let scheduler = windowed_ctx.animations.lock().unwrap();
                            let needs_animation_redraw = scheduler.take_needs_redraw();
                            drop(scheduler); // Release lock before request_redraw

                            // Check if stateful elements have active spring animations
                            // If so, re-run their callbacks to get updated animation values
                            if needs_animation_redraw && blinc_layout::has_animating_statefuls() {
                                blinc_layout::check_stateful_animations();
                            }

                            // Check if text widgets need continuous redraws (cursor blink)
                            let needs_cursor_redraw = blinc_layout::widgets::take_needs_continuous_redraw();

                            // Check if motion animations are active (enter/exit animations)
                            let needs_motion_redraw = if let Some(ref rs) = render_state {
                                rs.has_active_motions()
                            } else {
                                false
                            };

                            // Check if overlays changed (modal opened/closed, toast appeared, etc.)
                            let needs_overlay_redraw = {
                                let mgr = windowed_ctx.overlay_manager.lock().unwrap();
                                mgr.take_dirty() || mgr.has_visible_overlays()
                            };

                            // Check if CSS animations/transitions need continued redraws
                            // (includes transitions created during apply_complex_selector_styles)
                            let css_needs_redraw = css_active
                                || !render_tree
                                    .as_ref()
                                    .map_or(true, |t| t.css_transitions_empty());

                            let needs_e2e_redraw = e2e_enabled
                                && e2e_captures_done < e2e_max_captures
                                && ((e2e_capture_on_start && e2e_captures_done == 0)
                                    || e2e_trigger_path.is_some());

                            if needs_animation_redraw
                                || needs_cursor_redraw
                                || needs_motion_redraw
                                || scroll_animating
                                || needs_overlay_redraw
                                || theme_animating
                                || css_needs_redraw
                                || needs_e2e_redraw
                            {
                                // Request another frame to render updated animation values
                                // For cursor blink, also re-request continuous redraw for next frame
                                if needs_cursor_redraw {
                                    // Keep requesting redraws as long as a text input is focused
                                    if blinc_layout::widgets::has_focused_text_input() {
                                        blinc_layout::widgets::text_input::request_continuous_redraw_pub();
                                    }
                                }
                                window.request_redraw();
                            }
                        }
                    }

                    _ => {}
                }

                ControlFlow::Continue
            })
            .map_err(|e| BlincError::Platform(e.to_string()))?;

        Ok(())
    }

    /// Placeholder for non-windowed builds
    #[cfg(not(feature = "windowed"))]
    pub fn run<F, E>(_config: WindowConfig, _ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder + 'static,
    {
        Err(BlincError::Platform(
            "Windowed feature not enabled. Add 'windowed' feature to blinc_app".to_string(),
        ))
    }
}

/// Convert platform mouse button to layout mouse button
#[cfg(all(feature = "windowed", not(target_os = "android")))]
fn convert_mouse_button(button: blinc_platform::MouseButton) -> MouseButton {
    match button {
        blinc_platform::MouseButton::Left => MouseButton::Left,
        blinc_platform::MouseButton::Right => MouseButton::Right,
        blinc_platform::MouseButton::Middle => MouseButton::Middle,
        blinc_platform::MouseButton::Back => MouseButton::Back,
        blinc_platform::MouseButton::Forward => MouseButton::Forward,
        blinc_platform::MouseButton::Other(n) => MouseButton::Other(n),
    }
}

/// Convert layout cursor style to platform cursor
#[cfg(all(feature = "windowed", not(target_os = "android")))]
fn convert_cursor_style(cursor: CursorStyle) -> blinc_platform::Cursor {
    match cursor {
        CursorStyle::Default => blinc_platform::Cursor::Default,
        CursorStyle::Pointer => blinc_platform::Cursor::Pointer,
        CursorStyle::Text => blinc_platform::Cursor::Text,
        CursorStyle::Crosshair => blinc_platform::Cursor::Crosshair,
        CursorStyle::Move => blinc_platform::Cursor::Move,
        CursorStyle::NotAllowed => blinc_platform::Cursor::NotAllowed,
        CursorStyle::ResizeNS => blinc_platform::Cursor::ResizeNS,
        CursorStyle::ResizeEW => blinc_platform::Cursor::ResizeEW,
        CursorStyle::ResizeNESW => blinc_platform::Cursor::ResizeNESW,
        CursorStyle::ResizeNWSE => blinc_platform::Cursor::ResizeNWSE,
        CursorStyle::Grab => blinc_platform::Cursor::Grab,
        CursorStyle::Grabbing => blinc_platform::Cursor::Grabbing,
        CursorStyle::Wait => blinc_platform::Cursor::Wait,
        CursorStyle::Progress => blinc_platform::Cursor::Progress,
        CursorStyle::None => blinc_platform::Cursor::None,
    }
}

/// Convenience function to run a windowed app with default configuration
#[cfg(all(feature = "windowed", not(target_os = "android")))]
pub fn run_windowed<F, E>(ui_builder: F) -> Result<()>
where
    F: FnMut(&mut WindowedContext) -> E + 'static,
    E: ElementBuilder + 'static,
{
    WindowedApp::run(WindowConfig::default(), ui_builder)
}

/// Convenience function to run a windowed app with a title
#[cfg(all(feature = "windowed", not(target_os = "android")))]
pub fn run_windowed_with_title<F, E>(title: &str, ui_builder: F) -> Result<()>
where
    F: FnMut(&mut WindowedContext) -> E + 'static,
    E: ElementBuilder + 'static,
{
    let config = WindowConfig {
        title: title.to_string(),
        ..Default::default()
    };
    WindowedApp::run(config, ui_builder)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_ctx() -> WindowedContext {
        let animations: SharedAnimationScheduler = Arc::new(Mutex::new(AnimationScheduler::new()));
        let reactive: SharedReactiveGraph = Arc::new(Mutex::new(ReactiveGraph::new()));
        let hooks: SharedHookState = Arc::new(Mutex::new(HookState::new()));
        let ref_dirty_flag: RefDirtyFlag = Arc::new(AtomicBool::new(false));
        let element_registry: SharedElementRegistry =
            Arc::new(blinc_layout::selector::ElementRegistry::new());
        let ready_callbacks: SharedReadyCallbacks = Arc::new(Mutex::new(Vec::new()));

        WindowedContext {
            width: 100.0,
            height: 100.0,
            scale_factor: 1.0,
            physical_width: 100.0,
            physical_height: 100.0,
            focused: true,
            rebuild_count: 0,
            event_router: EventRouter::new(),
            animations,
            ref_dirty_flag,
            reactive,
            hooks,
            overlay_manager: overlay_manager(),
            had_visible_overlays: false,
            element_registry,
            ready_callbacks,
            stylesheet: None,
        }
    }

    #[test]
    fn test_use_state_keyed_init_can_call_use_state_keyed_without_deadlock() {
        let ctx = make_test_ctx();

        let outer: State<i32> = ctx.use_state_keyed("outer", || {
            let inner: State<i32> = ctx.use_state_keyed("inner", || 10);
            inner.get() + 5
        });

        assert_eq!(outer.get(), 15);
    }

    #[test]
    fn test_use_signal_keyed_init_can_call_use_state_keyed_without_deadlock() {
        let ctx = make_test_ctx();

        let sig: Signal<i32> = ctx.use_signal_keyed("sig", || {
            let inner: State<i32> = ctx.use_state_keyed("inner2", || 7);
            inner.get() * 2
        });

        assert_eq!(ctx.get(sig), Some(14));
    }

    #[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
    #[test]
    fn test_apply_navigation_policy_allows_configured_origin() {
        let config = WebViewConfig::new()
            .navigation_policy(WebViewNavigationPolicy::new().allow_origin("https://example.com"));

        let (config, decision) = apply_navigation_policy(config, Some("https://example.com/docs"));

        assert_eq!(
            config.initial_url.as_deref(),
            Some("https://example.com/docs")
        );
        assert_eq!(
            decision,
            Some(WebViewNavigationDecision::Allowed {
                url: "https://example.com/docs".to_string(),
                origin: "https://example.com".to_string(),
            })
        );
    }

    #[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
    #[test]
    fn test_apply_navigation_policy_blocks_disallowed_origin() {
        let config = WebViewConfig::new()
            .navigation_policy(WebViewNavigationPolicy::new().allow_origin("https://example.com"));

        let (config, decision) =
            apply_navigation_policy(config, Some("https://not-allowed.example/path"));

        assert_eq!(config.initial_url, None);
        assert_eq!(
            decision,
            Some(WebViewNavigationDecision::Blocked {
                url: "https://not-allowed.example/path".to_string(),
                origin: Some("https://not-allowed.example".to_string()),
                reason: WebViewNavigationBlockReason::OriginNotAllowed,
            })
        );
    }

    #[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
    #[test]
    fn test_apply_navigation_policy_blocks_malformed_url() {
        let config = WebViewConfig::new()
            .navigation_policy(WebViewNavigationPolicy::new().allow_origin("https://example.com"));

        let (config, decision) = apply_navigation_policy(config, Some("not-a-valid-url"));

        assert_eq!(config.initial_url, None);
        assert_eq!(
            decision,
            Some(WebViewNavigationDecision::Blocked {
                url: "not-a-valid-url".to_string(),
                origin: None,
                reason: WebViewNavigationBlockReason::MalformedUrl,
            })
        );
    }

    #[cfg(all(feature = "windowed", feature = "webview", not(target_os = "android")))]
    #[test]
    fn test_navigation_decision_event_names_are_machine_grepable() {
        let allowed = WebViewNavigationDecision::Allowed {
            url: "https://example.com/docs".to_string(),
            origin: "https://example.com".to_string(),
        };
        let blocked = WebViewNavigationDecision::Blocked {
            url: "https://not-allowed.example".to_string(),
            origin: Some("https://not-allowed.example".to_string()),
            reason: WebViewNavigationBlockReason::OriginNotAllowed,
        };

        assert_eq!(
            navigation_decision_event_name(&allowed),
            "navigation allowed"
        );
        assert_eq!(
            navigation_decision_event_name(&blocked),
            "navigation blocked"
        );
    }
}
