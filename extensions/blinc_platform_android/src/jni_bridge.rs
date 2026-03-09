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
use tracing::{debug, error, info, warn};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeLifecycleState {
    Foreground,
    Inactive,
    Background,
    Destroyed,
}

impl RuntimeLifecycleState {
    fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Foreground,
            1 => Self::Inactive,
            2 => Self::Background,
            3 => Self::Destroyed,
            _ => Self::Inactive,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EmbeddingRuntimeState {
    width: u32,
    height: u32,
    scale_factor: f64,
    surface_attached: bool,
    destroyed: bool,
    lifecycle: RuntimeLifecycleState,
}

impl EmbeddingRuntimeState {
    fn new(width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            width,
            height,
            scale_factor,
            surface_attached: true,
            destroyed: false,
            lifecycle: RuntimeLifecycleState::Foreground,
        }
    }

    fn attach_surface(
        &mut self,
        width: u32,
        height: u32,
        scale_factor: f64,
    ) -> Result<(), &'static str> {
        if self.destroyed {
            return Err("runtime destroyed");
        }
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        self.surface_attached = true;
        Ok(())
    }

    fn detach_surface(&mut self) -> Result<(), &'static str> {
        if self.destroyed {
            return Err("runtime destroyed");
        }
        self.surface_attached = false;
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), &'static str> {
        if self.destroyed {
            return Err("runtime destroyed");
        }
        if !self.surface_attached {
            return Err("surface detached");
        }
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn on_lifecycle(&mut self, lifecycle: RuntimeLifecycleState) -> Result<(), &'static str> {
        if self.destroyed && lifecycle != RuntimeLifecycleState::Destroyed {
            return Err("runtime destroyed");
        }
        self.lifecycle = lifecycle;
        if lifecycle == RuntimeLifecycleState::Destroyed {
            self.destroyed = true;
            self.surface_attached = false;
        }
        Ok(())
    }

    fn render_allowed(&self) -> bool {
        !self.destroyed
            && self.surface_attached
            && matches!(
                self.lifecycle,
                RuntimeLifecycleState::Foreground | RuntimeLifecycleState::Inactive
            )
    }
}

fn commit_surface_attach(
    runtime: &mut EmbeddingRuntimeState,
    current_window_ptr: &mut *mut std::ffi::c_void,
    next_window_ptr: *mut std::ffi::c_void,
    width: u32,
    height: u32,
    scale_factor: f64,
) -> Result<Option<*mut std::ffi::c_void>, &'static str> {
    runtime.attach_surface(width, height, scale_factor)?;
    let previous = if current_window_ptr.is_null() {
        None
    } else {
        Some(*current_window_ptr)
    };
    *current_window_ptr = next_window_ptr;
    Ok(previous)
}

/// Opaque handle to BlincRenderer state
/// This is passed to Kotlin as a Long and cast back when needed
#[cfg(target_os = "android")]
struct BlincHandle {
    runtime: EmbeddingRuntimeState,
    /// Width of the surface in pixels
    width: u32,
    /// Height of the surface in pixels
    height: u32,
    /// Scale factor (display density)
    scale_factor: f64,
    /// Whether touch is currently active
    touch_active: bool,
    /// Last touch position
    last_touch: (f32, f32),
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
            runtime: EmbeddingRuntimeState::new(width, height, scale_factor),
            width,
            height,
            scale_factor,
            touch_active: false,
            last_touch: (0.0, 0.0),
            native_window_ptr,
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
    let handle = Box::new(BlincHandle::new(
        width as u32,
        height as u32,
        scale_factor,
        native_window_ptr,
    ));

    // TODO: Initialize GPU renderer with native_window_ptr
    // This would involve:
    // 1. Create wgpu Instance with Vulkan backend
    // 2. Create surface from native window pointer
    // 3. Initialize GpuRenderer
    // 4. Store in handle

    // Convert to raw pointer and return as jlong
    let ptr = Box::into_raw(handle);
    info!("Created BlincHandle at {:p}", ptr);

    ptr as jlong
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

    let _blinc = unsafe { &mut *(handle as *mut BlincHandle) };

    if !_blinc.runtime.render_allowed() {
        debug!("nativeRenderFrame skipped: runtime not renderable");
        return;
    }

    // TODO: Implement actual rendering
    // 1. Get surface texture
    // 2. Clear with background color
    // 3. Render UI tree
    // 4. Present

    debug!("nativeRenderFrame called");
}

/// Attach or re-attach a Surface to an existing runtime.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeAttachSurface(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    surface: JObject,
    width: jint,
    height: jint,
    density: jfloat,
) -> jboolean {
    if handle == 0 || width <= 0 || height <= 0 {
        return JNI_FALSE;
    }

    let blinc = unsafe { &mut *(handle as *mut BlincHandle) };
    let native_window_ptr = match get_native_window_from_surface(&mut env, &surface) {
        Ok(ptr) => ptr,
        Err(err) => {
            error!("nativeAttachSurface failed to get surface: {}", err);
            return JNI_FALSE;
        }
    };

    let next_width = width as u32;
    let next_height = height as u32;
    let next_scale_factor = if density > 0.0 { density as f64 } else { 1.0 };
    match commit_surface_attach(
        &mut blinc.runtime,
        &mut blinc.native_window_ptr,
        native_window_ptr,
        next_width,
        next_height,
        next_scale_factor,
    ) {
        Ok(previous_window_ptr) => {
            blinc.width = next_width;
            blinc.height = next_height;
            blinc.scale_factor = next_scale_factor;
            if let Some(previous_window_ptr) = previous_window_ptr {
                unsafe { ANativeWindow_release(previous_window_ptr) };
            }
            JNI_TRUE
        }
        Err(err) => {
            unsafe { ANativeWindow_release(native_window_ptr) };
            error!("nativeAttachSurface rejected: {}", err);
            JNI_FALSE
        }
    }
}

/// Detach the current Surface from an existing runtime without destroying it.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeDetachSurface(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }

    let blinc = unsafe { &mut *(handle as *mut BlincHandle) };
    if let Err(err) = blinc.runtime.detach_surface() {
        warn!("nativeDetachSurface ignored: {}", err);
        return;
    }
    if !blinc.native_window_ptr.is_null() {
        unsafe { ANativeWindow_release(blinc.native_window_ptr) };
        blinc.native_window_ptr = std::ptr::null_mut();
    }
}

/// Update lifecycle state for the embedded runtime.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_blinc_BlincBridge_nativeOnLifecycle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    state: jint,
) {
    if handle == 0 {
        return;
    }

    let blinc = unsafe { &mut *(handle as *mut BlincHandle) };
    if let Err(err) = blinc
        .runtime
        .on_lifecycle(RuntimeLifecycleState::from_raw(state))
    {
        warn!("nativeOnLifecycle ignored: {}", err);
    }
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

    let blinc = unsafe { &mut *(handle as *mut BlincHandle) };

    // Convert to logical coordinates
    let logical_x = x / blinc.scale_factor as f32;
    let logical_y = y / blinc.scale_factor as f32;

    // Android MotionEvent actions
    const ACTION_DOWN: i32 = 0;
    const ACTION_UP: i32 = 1;
    const ACTION_MOVE: i32 = 2;
    const ACTION_CANCEL: i32 = 3;

    match action {
        ACTION_DOWN => {
            debug!("Touch down at ({}, {})", logical_x, logical_y);
            blinc.touch_active = true;
            blinc.last_touch = (logical_x, logical_y);
            // TODO: Route to event router
        }
        ACTION_UP => {
            debug!("Touch up at ({}, {})", logical_x, logical_y);
            blinc.touch_active = false;
            blinc.last_touch = (logical_x, logical_y);
            // TODO: Route to event router
        }
        ACTION_MOVE => {
            if blinc.touch_active {
                debug!("Touch move to ({}, {})", logical_x, logical_y);
                blinc.last_touch = (logical_x, logical_y);
                // TODO: Route to event router
            }
        }
        ACTION_CANCEL => {
            debug!("Touch cancelled");
            blinc.touch_active = false;
            // TODO: Route to event router
        }
        _ => {
            debug!("Unknown touch action: {}", action);
        }
    }

    JNI_TRUE
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

    let blinc = unsafe { &mut *(handle as *mut BlincHandle) };

    info!("Surface resized to {}x{}", width, height);
    match blinc.runtime.resize(width as u32, height as u32) {
        Ok(()) => {
            blinc.width = width as u32;
            blinc.height = height as u32;
        }
        Err(err) => {
            warn!("nativeResize ignored: {}", err);
            return;
        }
    }

    // TODO: Reconfigure wgpu surface with new dimensions
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

    info!("Destroying BlincHandle at {:p}", handle as *const ());

    // Reclaim the Box and drop it
    let blinc = unsafe { Box::from_raw(handle as *mut BlincHandle) };
    let mut runtime = blinc.runtime;
    let _ = runtime.on_lifecycle(RuntimeLifecycleState::Destroyed);

    // Release the native window reference
    if !blinc.native_window_ptr.is_null() {
        unsafe { ANativeWindow_release(blinc.native_window_ptr) };
    }

    // TODO: Clean up GPU resources
    // The Box will be dropped here, cleaning up the handle

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
    use super::{commit_surface_attach, EmbeddingRuntimeState, RuntimeLifecycleState};

    #[test]
    fn embedding_runtime_state_tracks_attach_detach_and_destroy() {
        let mut state = EmbeddingRuntimeState::new(100, 200, 2.0);
        assert!(state.render_allowed());

        state.detach_surface().expect("detach");
        assert!(!state.render_allowed());

        state.attach_surface(320, 640, 3.0).expect("attach");
        assert!(state.render_allowed());
        assert_eq!(state.width, 320);
        assert_eq!(state.height, 640);
        assert_eq!(state.scale_factor, 3.0);

        state
            .on_lifecycle(RuntimeLifecycleState::Background)
            .expect("background");
        assert!(!state.render_allowed());

        state
            .on_lifecycle(RuntimeLifecycleState::Destroyed)
            .expect("destroy");
        assert!(state.destroyed);
        assert!(!state.surface_attached);
        assert!(!state.render_allowed());
    }

    #[test]
    fn runtime_lifecycle_state_maps_from_jni_values() {
        assert_eq!(
            RuntimeLifecycleState::from_raw(0),
            RuntimeLifecycleState::Foreground
        );
        assert_eq!(
            RuntimeLifecycleState::from_raw(1),
            RuntimeLifecycleState::Inactive
        );
        assert_eq!(
            RuntimeLifecycleState::from_raw(2),
            RuntimeLifecycleState::Background
        );
        assert_eq!(
            RuntimeLifecycleState::from_raw(3),
            RuntimeLifecycleState::Destroyed
        );
        assert_eq!(
            RuntimeLifecycleState::from_raw(99),
            RuntimeLifecycleState::Inactive
        );
    }

    #[test]
    fn failed_surface_attach_does_not_replace_current_window_pointer() {
        let mut runtime = EmbeddingRuntimeState::new(100, 200, 2.0);
        runtime
            .on_lifecycle(RuntimeLifecycleState::Destroyed)
            .expect("destroy");
        let original = 0x10usize as *mut std::ffi::c_void;
        let mut current = original;
        let next = 0x20usize as *mut std::ffi::c_void;

        let result = commit_surface_attach(&mut runtime, &mut current, next, 320, 640, 3.0);
        assert_eq!(result, Err("runtime destroyed"));
        assert_eq!(current, original);
    }
}
