//! Kotlin JNI Bridge for Blinc
//!
//! Provides JNI functions for embedding Blinc rendering into Kotlin/Java Android applications.
//! This allows developers to use Blinc as a rendering engine within existing Android apps.
//!
//! # Usage from Kotlin
//!
//! ```kotlin
//! package com.blinc
//!
//! import android.view.Surface
//!
//! object BlincBridge {
//!     init {
//!         System.loadLibrary("blinc_platform_android")
//!     }
//!
//!     external fun nativeInit(surface: Surface, width: Int, height: Int, density: Float): Long
//!     external fun nativeRenderFrame(handle: Long)
//!     external fun nativeOnTouch(handle: Long, action: Int, x: Float, y: Float): Boolean
//!     external fun nativeResize(handle: Long, width: Int, height: Int)
//!     external fun nativeDestroy(handle: Long)
//! }
//! ```
//!
//! # Example Usage
//!
//! ```kotlin
//! class BlincSurfaceView(context: Context) : SurfaceView(context), SurfaceHolder.Callback {
//!     private var blincHandle: Long = 0
//!
//!     init {
//!         holder.addCallback(this)
//!     }
//!
//!     override fun surfaceCreated(holder: SurfaceHolder) {
//!         val metrics = resources.displayMetrics
//!         blincHandle = BlincBridge.nativeInit(
//!             holder.surface,
//!             holder.surfaceFrame.width(),
//!             holder.surfaceFrame.height(),
//!             metrics.density
//!         )
//!     }
//!
//!     override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
//!         if (blincHandle != 0L) {
//!             BlincBridge.nativeResize(blincHandle, width, height)
//!         }
//!     }
//!
//!     override fun surfaceDestroyed(holder: SurfaceHolder) {
//!         if (blincHandle != 0L) {
//!             BlincBridge.nativeDestroy(blincHandle)
//!             blincHandle = 0
//!         }
//!     }
//!
//!     fun render() {
//!         if (blincHandle != 0L) {
//!             BlincBridge.nativeRenderFrame(blincHandle)
//!         }
//!     }
//! }
//! ```

#[cfg(target_os = "android")]
use jni::objects::{JClass, JObject};
#[cfg(target_os = "android")]
use jni::sys::{jboolean, jfloat, jint, jlong, JNI_FALSE, JNI_TRUE};
#[cfg(target_os = "android")]
use jni::JNIEnv;

#[cfg(target_os = "android")]
use ndk::native_window::NativeWindow;

#[cfg(target_os = "android")]
use std::collections::HashMap;

#[cfg(target_os = "android")]
use std::sync::atomic::{AtomicI64, Ordering};

#[cfg(target_os = "android")]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(target_os = "android")]
use tracing::{debug, error, info, warn};

const MAX_QUEUED_TOUCH_EVENTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TouchPhase {
    Down,
    Up,
    Move,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TouchEventRecord {
    phase: TouchPhase,
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct BlincRuntimeState {
    width: u32,
    height: u32,
    scale_factor: f64,
    focused: bool,
    redraw_requested: bool,
    surface_dirty: bool,
    touch_active: bool,
    last_touch: (f32, f32),
    queued_touch_events: Vec<TouchEventRecord>,
    last_rendered_size: Option<(u32, u32)>,
}

impl BlincRuntimeState {
    fn new(width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            width,
            height,
            scale_factor,
            focused: true,
            redraw_requested: true,
            surface_dirty: true,
            touch_active: false,
            last_touch: (0.0, 0.0),
            queued_touch_events: Vec::new(),
            last_rendered_size: None,
        }
    }

    fn push_touch_event(&mut self, event: TouchEventRecord) {
        if self.queued_touch_events.len() == MAX_QUEUED_TOUCH_EVENTS {
            self.queued_touch_events.remove(0);
        }
        self.queued_touch_events.push(event);
    }

    fn logical_point(&self, x: f32, y: f32) -> (f32, f32) {
        let scale = self.scale_factor.max(f64::EPSILON) as f32;
        (x / scale, y / scale)
    }

    fn record_touch(&mut self, phase: TouchPhase, x: f32, y: f32) -> bool {
        let (logical_x, logical_y) = self.logical_point(x, y);
        match phase {
            TouchPhase::Down => {
                self.touch_active = true;
                self.last_touch = (logical_x, logical_y);
                self.push_touch_event(TouchEventRecord {
                    phase,
                    x: logical_x,
                    y: logical_y,
                });
                self.redraw_requested = true;
                true
            }
            TouchPhase::Up => {
                self.touch_active = false;
                self.last_touch = (logical_x, logical_y);
                self.push_touch_event(TouchEventRecord {
                    phase,
                    x: logical_x,
                    y: logical_y,
                });
                self.redraw_requested = true;
                true
            }
            TouchPhase::Move => {
                if !self.touch_active {
                    return false;
                }
                self.last_touch = (logical_x, logical_y);
                self.push_touch_event(TouchEventRecord {
                    phase,
                    x: logical_x,
                    y: logical_y,
                });
                self.redraw_requested = true;
                true
            }
            TouchPhase::Cancel => {
                self.touch_active = false;
                self.push_touch_event(TouchEventRecord {
                    phase,
                    x: logical_x,
                    y: logical_y,
                });
                self.redraw_requested = true;
                true
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.surface_dirty = true;
        self.redraw_requested = true;
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if focused {
            self.redraw_requested = true;
        }
    }

    fn note_rendered(&mut self) {
        self.last_rendered_size = Some((self.width, self.height));
        self.redraw_requested = false;
        self.surface_dirty = false;
    }

    fn take_touch_events(&mut self) -> Vec<TouchEventRecord> {
        std::mem::take(&mut self.queued_touch_events)
    }
}

/// Opaque handle to BlincRenderer state
/// This is passed to Kotlin as a Long and cast back when needed
#[cfg(target_os = "android")]
struct BlincHandle {
    runtime: BlincRuntimeState,
    /// Native window pointer for GPU rendering
    native_window_ptr: *mut std::ffi::c_void,
    // TODO: Add actual renderer state when blinc_gpu is integrated
    // renderer: Option<BlincApp>,
    // surface: Option<wgpu::Surface>,
}

#[cfg(target_os = "android")]
impl BlincHandle {
    fn new(
        width: u32,
        height: u32,
        scale_factor: f64,
        native_window_ptr: *mut std::ffi::c_void,
    ) -> Self {
        Self {
            runtime: BlincRuntimeState::new(width, height, scale_factor),
            native_window_ptr,
        }
    }
}

#[cfg(target_os = "android")]
fn next_handle_id() -> i64 {
    static NEXT_HANDLE_ID: AtomicI64 = AtomicI64::new(1);
    NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(target_os = "android")]
fn handle_registry() -> &'static Mutex<HashMap<i64, Arc<Mutex<BlincHandle>>>> {
    static HANDLE_REGISTRY: OnceLock<Mutex<HashMap<i64, Arc<Mutex<BlincHandle>>>>> =
        OnceLock::new();
    HANDLE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(target_os = "android")]
fn lock_or_recover<'a, T>(mutex: &'a Mutex<T>, label: &str) -> std::sync::MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Recovering from poisoned mutex: {}", label);
            poisoned.into_inner()
        }
    }
}

#[cfg(target_os = "android")]
fn register_handle(handle: BlincHandle) -> i64 {
    let id = next_handle_id();
    let mut registry = lock_or_recover(handle_registry(), "android handle registry");
    registry.insert(id, Arc::new(Mutex::new(handle)));
    id
}

#[cfg(target_os = "android")]
fn get_handle(handle: jlong) -> Option<Arc<Mutex<BlincHandle>>> {
    let id = i64::try_from(handle).ok()?;
    let registry = lock_or_recover(handle_registry(), "android handle registry");
    registry.get(&id).cloned()
}

#[cfg(target_os = "android")]
fn destroy_handle(handle: jlong) -> Option<BlincHandle> {
    let id = i64::try_from(handle).ok()?;
    let arc = {
        let mut registry = lock_or_recover(handle_registry(), "android handle registry");
        registry.remove(&id)?
    };

    match Arc::try_unwrap(arc) {
        Ok(mutex) => match mutex.into_inner() {
            Ok(handle) => Some(handle),
            Err(poisoned) => {
                warn!("Recovering from poisoned BlincHandle during destroy");
                Some(poisoned.into_inner())
            }
        },
        Err(arc) => {
            let mut guard = lock_or_recover(&arc, "android blinc handle");
            Some(std::mem::replace(
                &mut *guard,
                BlincHandle::new(0, 0, 1.0, std::ptr::null_mut()),
            ))
        }
    }
}

/// Initialize Blinc renderer with an Android Surface
///
/// # Arguments
/// * `surface` - Android Surface object from SurfaceView or TextureView
/// * `width` - Surface width in pixels (from Kotlin)
/// * `height` - Surface height in pixels (from Kotlin)
/// * `density` - Display density from DisplayMetrics (from Kotlin)
///
/// # Returns
/// * Opaque handle (Long) to the renderer, or 0 on failure
///
/// # JNI Signature
/// `(Landroid/view/Surface;IIF)J`
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    surface: JObject,
    width: jint,
    height: jint,
    density: jfloat,
) -> jlong {
    // Initialize Android logging if not already done
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("BlincJNI"),
    );

    info!("BlincBridge.nativeInit called");
    info!(
        "Surface dimensions: {}x{}, density: {}",
        width, height, density
    );

    // Validate parameters
    if width <= 0 || height <= 0 {
        error!("Invalid surface dimensions: {}x{}", width, height);
        return 0;
    }

    let scale_factor = if density > 0.0 { density as f64 } else { 1.0 };

    // Get ANativeWindow from Surface for GPU rendering
    let native_window_ptr = match get_native_window_from_surface(&mut env, &surface) {
        Ok(ptr) => ptr,
        Err(e) => {
            error!("Failed to get native window: {}", e);
            return 0;
        }
    };

    // Create handle with surface info
    let handle_id = register_handle(BlincHandle::new(
        width as u32,
        height as u32,
        scale_factor,
        native_window_ptr,
    ));

    info!("Created BlincHandle id {}", handle_id);

    handle_id as jlong
}

/// Render a frame
///
/// # Arguments
/// * `handle` - Opaque handle from nativeInit
///
/// # JNI Signature
/// `(J)V`
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeRenderFrame(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle == 0 {
        warn!("nativeRenderFrame called with null handle");
        return;
    }

    let arc = match get_handle(handle) {
        Some(arc) => arc,
        None => {
            warn!("nativeRenderFrame called with unknown or destroyed handle");
            return;
        }
    };
    let mut blinc = lock_or_recover(&arc, "android blinc handle");

    let was_dirty = blinc.runtime.surface_dirty;
    let was_redraw_requested = blinc.runtime.redraw_requested;
    blinc.runtime.note_rendered();

    debug!(
        "nativeRenderFrame called for handle {} at {}x{} (dirty={}, redraw_requested={}, focused={})",
        handle,
        blinc.runtime.width,
        blinc.runtime.height,
        was_dirty,
        was_redraw_requested,
        blinc.runtime.focused
    );
}

/// Handle touch input event
///
/// # Arguments
/// * `handle` - Opaque handle from nativeInit
/// * `action` - MotionEvent action (ACTION_DOWN=0, ACTION_UP=1, ACTION_MOVE=2, etc.)
/// * `x` - Touch X coordinate in pixels
/// * `y` - Touch Y coordinate in pixels
///
/// # Returns
/// * true if the event was handled
///
/// # JNI Signature
/// `(JIFF)Z`
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeOnTouch(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    action: jint,
    x: jfloat,
    y: jfloat,
) -> jboolean {
    if handle == 0 {
        warn!("nativeOnTouch called with null handle");
        return JNI_FALSE;
    }

    let arc = match get_handle(handle) {
        Some(arc) => arc,
        None => {
            warn!("nativeOnTouch called with unknown or destroyed handle");
            return JNI_FALSE;
        }
    };
    let mut blinc = lock_or_recover(&arc, "android blinc handle");

    // Android MotionEvent actions
    const ACTION_DOWN: i32 = 0;
    const ACTION_UP: i32 = 1;
    const ACTION_MOVE: i32 = 2;
    const ACTION_CANCEL: i32 = 3;

    let handled = match action {
        ACTION_DOWN => {
            let handled = blinc.runtime.record_touch(TouchPhase::Down, x, y);
            debug!("Touch down handled={}", handled);
            handled
        }
        ACTION_UP => {
            let handled = blinc.runtime.record_touch(TouchPhase::Up, x, y);
            debug!("Touch up handled={}", handled);
            handled
        }
        ACTION_MOVE => {
            let handled = blinc.runtime.record_touch(TouchPhase::Move, x, y);
            debug!("Touch move handled={}", handled);
            handled
        }
        ACTION_CANCEL => {
            debug!("Touch cancelled");
            blinc.runtime.record_touch(TouchPhase::Cancel, x, y)
        }
        _ => {
            debug!("Unknown touch action: {}", action);
            return JNI_FALSE;
        }
    };

    if handled {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// Handle surface resize
///
/// # Arguments
/// * `handle` - Opaque handle from nativeInit
/// * `width` - New width in pixels
/// * `height` - New height in pixels
///
/// # JNI Signature
/// `(JII)V`
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeResize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
) {
    if handle == 0 {
        warn!("nativeResize called with null handle");
        return;
    }
    let arc = match get_handle(handle) {
        Some(arc) => arc,
        None => {
            warn!("nativeResize called with unknown or destroyed handle");
            return;
        }
    };
    let mut blinc = lock_or_recover(&arc, "android blinc handle");

    if width <= 0 || height <= 0 {
        warn!("Ignoring invalid resize to {}x{}", width, height);
        return;
    }

    info!("Surface resized to {}x{}", width, height);
    blinc.runtime.resize(width as u32, height as u32);
}

/// Destroy the renderer and free resources
///
/// # Arguments
/// * `handle` - Opaque handle from nativeInit
///
/// # JNI Signature
/// `(J)V`
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle == 0 {
        warn!("nativeDestroy called with null handle");
        return;
    }

    info!("Destroying BlincHandle id {}", handle);

    let Some(blinc) = destroy_handle(handle) else {
        warn!("nativeDestroy called with unknown or already-destroyed handle");
        return;
    };

    // Release the native window reference
    if !blinc.native_window_ptr.is_null() {
        unsafe { ANativeWindow_release(blinc.native_window_ptr) };
    }

    // TODO: Clean up GPU resources

    info!("BlincHandle destroyed");
}

// ============================================================================
// Helper functions
// ============================================================================

// FFI declaration for ANativeWindow_fromSurface
#[cfg(target_os = "android")]
extern "C" {
    fn ANativeWindow_fromSurface(
        env: *mut std::ffi::c_void,
        surface: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;

    fn ANativeWindow_release(window: *mut std::ffi::c_void);
}

/// Get ANativeWindow from android.view.Surface via NDK
#[cfg(target_os = "android")]
fn get_native_window_from_surface(
    env: &mut JNIEnv,
    surface: &JObject,
) -> Result<*mut std::ffi::c_void, String> {
    let surface_ptr = surface.as_raw();

    // Get the native window using NDK function
    let native_window = unsafe {
        ANativeWindow_fromSurface(
            env.get_raw() as *mut std::ffi::c_void,
            surface_ptr as *mut _,
        )
    };

    if native_window.is_null() {
        return Err("ANativeWindow_fromSurface returned null".to_string());
    }

    Ok(native_window)
}

// ============================================================================
// Non-Android stubs
// ============================================================================

#[cfg(not(target_os = "android"))]
pub fn jni_bridge_placeholder() {
    // Placeholder for non-Android builds
}

#[cfg(test)]
mod tests {
    use super::{BlincRuntimeState, TouchPhase, MAX_QUEUED_TOUCH_EVENTS};

    #[test]
    fn runtime_starts_dirty_and_focused() {
        let runtime = BlincRuntimeState::new(1080, 2400, 3.0);
        assert!(runtime.focused);
        assert!(runtime.redraw_requested);
        assert!(runtime.surface_dirty);
        assert_eq!(runtime.last_rendered_size, None);
    }

    #[test]
    fn resize_marks_surface_dirty_and_requests_redraw() {
        let mut runtime = BlincRuntimeState::new(100, 200, 2.0);
        runtime.note_rendered();
        runtime.resize(300, 400);

        assert_eq!((runtime.width, runtime.height), (300, 400));
        assert!(runtime.redraw_requested);
        assert!(runtime.surface_dirty);
    }

    #[test]
    fn touch_events_are_recorded_in_logical_coordinates() {
        let mut runtime = BlincRuntimeState::new(1080, 1920, 2.0);

        assert!(runtime.record_touch(TouchPhase::Down, 200.0, 300.0));
        assert!(runtime.record_touch(TouchPhase::Move, 240.0, 320.0));
        assert!(runtime.record_touch(TouchPhase::Up, 240.0, 320.0));

        let events = runtime.take_touch_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].x, 100.0);
        assert_eq!(events[0].y, 150.0);
        assert_eq!(events[1].phase, TouchPhase::Move);
        assert_eq!(events[2].phase, TouchPhase::Up);
    }

    #[test]
    fn move_without_active_touch_is_ignored() {
        let mut runtime = BlincRuntimeState::new(100, 100, 1.0);
        assert!(!runtime.record_touch(TouchPhase::Move, 10.0, 20.0));
        assert!(runtime.take_touch_events().is_empty());
    }

    #[test]
    fn render_clears_dirty_flags_and_records_size() {
        let mut runtime = BlincRuntimeState::new(640, 480, 1.0);
        runtime.note_rendered();

        assert!(!runtime.redraw_requested);
        assert!(!runtime.surface_dirty);
        assert_eq!(runtime.last_rendered_size, Some((640, 480)));
    }

    #[test]
    fn touch_queue_is_bounded() {
        let mut runtime = BlincRuntimeState::new(100, 100, 1.0);
        assert!(runtime.record_touch(TouchPhase::Down, 0.0, 0.0));
        for i in 0..(MAX_QUEUED_TOUCH_EVENTS + 8) {
            assert!(runtime.record_touch(TouchPhase::Move, i as f32, i as f32));
        }

        let events = runtime.take_touch_events();
        assert_eq!(events.len(), MAX_QUEUED_TOUCH_EVENTS);
        assert_eq!(events.first().unwrap().phase, TouchPhase::Move);
    }
}
