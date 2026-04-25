/**
 * Blinc Native Bridge for Android
 *
 * Kotlin implementation for handling native calls from Rust.
 * Register handlers for each namespace/function, then Rust can call
 * them via native_call("namespace", "function", args).
 *
 * Usage:
 * ```kotlin
 * // In Application.onCreate()
 * BlincNativeBridge.registerDefaults(context)
 *
 * // Or register custom handlers
 * BlincNativeBridge.register("myapi", "my_function") { args ->
 *     // args is JSONArray
 *     "result"
 * }
 * ```
 */

package com.blinc

import android.app.Activity
import android.Manifest
import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.hardware.Sensor
import android.hardware.SensorEvent
import android.hardware.SensorEventListener
import android.hardware.SensorManager
import android.location.Location
import android.location.LocationListener
import android.location.LocationManager
import android.net.Uri
import android.os.BatteryManager
import android.os.Build
import android.os.Looper
import android.os.ParcelUuid
import android.os.SystemClock
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.view.inputmethod.InputMethodManager
import androidx.core.content.getSystemService
import org.json.JSONArray
import org.json.JSONObject
import java.util.ArrayDeque
import java.util.Locale
import java.util.TimeZone
import java.lang.ref.WeakReference

object BlincNativeBridge {
    private data class PermissionCapabilityState(
        val status: String,
        val canRequest: Boolean,
        val requiresSettingsRedirect: Boolean,
        val supported: Boolean = true,
    )


    // Handler type: (args: JSONArray) -> Any?
    private val handlers = mutableMapOf<String, MutableMap<String, (JSONArray) -> Any?>>()

    // Application context for system services
    private var appContext: Context? = null
    private var foregroundActivityRef: WeakReference<Activity>? = null
    private val sensorCollector = AndroidSensorCollector()
    private val bleCollector = AndroidBleCollector(sensorCollector)

    // Activity reference (when initialized from an Activity) — required for
    // anything that needs a window/decor view, e.g. soft keyboard show/hide.
    // The application context returned by `Context.applicationContext` is NOT
    // an Activity, so storing the original `Context` here lets us recover
    // the Activity when callers pass one in.
    private var activityRef: java.lang.ref.WeakReference<android.app.Activity>? = null

    // Last IME inset (in logical pixels) we pushed to Rust. Used by the
    // window-insets listener to skip duplicate dispatches when nothing
    // about the keyboard has actually changed.
    private var lastDispatchedImeInsetPx: Int = -1

    // Last system-bar safe-area insets (in logical pixels) pushed to Rust.
    // The single `setOnApplyWindowInsetsListener` on the decor view fires
    // for every inset type, so we share it between the IME path and the
    // notch / status-bar / nav-bar / gesture-bar path and dedupe each
    // stream independently.
    private var lastDispatchedSafeAreaTopPx: Int = -1
    private var lastDispatchedSafeAreaRightPx: Int = -1
    private var lastDispatchedSafeAreaBottomPx: Int = -1
    private var lastDispatchedSafeAreaLeftPx: Int = -1

    /**
     * Initialize with application context
     */
    fun init(context: Context) {
        appContext = context.applicationContext
        if (context is android.app.Activity) {
            activityRef = java.lang.ref.WeakReference(context)
            attachWindowInsetsListener(context)
        }
    }

    private fun currentActivity(): android.app.Activity? = activityRef?.get()

    /**
     * Wire up a shared window-insets listener on the activity's decor view.
     *
     * A single [View.setOnApplyWindowInsetsListener] callback carries every
     * kind of inset (IME, status bar, navigation bar, gesture bar, display
     * cutout, ...). We dispatch two streams from it:
     *
     *  1. **Soft-keyboard / IME inset** → mirrors iOS'
     *     `UIKeyboardWillChangeFrameNotification`. Whenever the keyboard
     *     bottom inset changes (show, hide, hardware-keyboard attach,
     *     split-keyboard mode, IME swap, ...) the listener computes the
     *     bottom inset in **logical pixels** and pushes it via
     *     [nativeDispatchKeyboardInset].
     *
     *  2. **System-bar safe-area insets** → mirrors iOS'
     *     `UIWindow.safeAreaInsets`. Whenever the status bar / nav bar /
     *     notch cutout / gesture bar insets change (rotation, split-screen,
     *     PiP exit, immersive-mode toggle, display cutout mode, ...) the
     *     listener converts the four edges to logical pixels and pushes
     *     them via [nativeDispatchSafeArea].
     *
     * Both streams dedupe against their respective `lastDispatched*Px`
     * fields so unrelated inset changes don't thrash the Rust side.
     *
     * Implementation notes:
     * - `WindowInsets.Type.ime()` and `WindowInsets.Type.systemBars()`
     *   both require API 30+. On older devices we fall back to the
     *   `systemWindowInset*` accessors (deprecated but functional on
     *   API 24–29) plus a global-layout listener for the IME path.
     */
    private fun attachWindowInsetsListener(activity: android.app.Activity) {
        val decorView = activity.window?.decorView ?: return
        val density = activity.resources.displayMetrics.density.coerceAtLeast(0.001f)

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            // Modern path (API 30+): WindowInsets.Type reports per-type
            // insets in physical pixels. The listener fires for every
            // animation frame as the keyboard slides in / out, and
            // again on rotation / split-screen transitions.
            decorView.setOnApplyWindowInsetsListener { v, insets ->
                // --- IME ---
                val imeBottomPx = insets.getInsets(android.view.WindowInsets.Type.ime()).bottom
                val imeLogicalPx = (imeBottomPx.toFloat() / density).toInt()
                if (imeLogicalPx != lastDispatchedImeInsetPx) {
                    lastDispatchedImeInsetPx = imeLogicalPx
                    try {
                        nativeDispatchKeyboardInset(imeLogicalPx)
                    } catch (e: UnsatisfiedLinkError) {
                        // Native side hasn't loaded the symbol yet — most
                        // likely because the user app isn't using
                        // blinc_app::android::AndroidApp::run. Skip
                        // silently; the inset just won't propagate.
                    }
                }

                // --- System bars (status / nav / cutout / gesture) ---
                val sys = insets.getInsets(android.view.WindowInsets.Type.systemBars())
                val cutout = insets.getInsets(android.view.WindowInsets.Type.displayCutout())
                // Merge system bars with the display cutout — the cutout
                // can extend past the status bar on landscape phones
                // with a camera notch.
                val topLogical = (maxOf(sys.top, cutout.top).toFloat() / density).toInt()
                val rightLogical = (maxOf(sys.right, cutout.right).toFloat() / density).toInt()
                val bottomLogical = (maxOf(sys.bottom, cutout.bottom).toFloat() / density).toInt()
                val leftLogical = (maxOf(sys.left, cutout.left).toFloat() / density).toInt()
                if (topLogical != lastDispatchedSafeAreaTopPx
                    || rightLogical != lastDispatchedSafeAreaRightPx
                    || bottomLogical != lastDispatchedSafeAreaBottomPx
                    || leftLogical != lastDispatchedSafeAreaLeftPx
                ) {
                    lastDispatchedSafeAreaTopPx = topLogical
                    lastDispatchedSafeAreaRightPx = rightLogical
                    lastDispatchedSafeAreaBottomPx = bottomLogical
                    lastDispatchedSafeAreaLeftPx = leftLogical
                    try {
                        nativeDispatchSafeArea(topLogical, rightLogical, bottomLogical, leftLogical)
                    } catch (e: UnsatisfiedLinkError) {
                        // see IME branch
                    }
                }

                v.onApplyWindowInsets(insets)
            }
            // Force an initial dispatch so we have a baseline (otherwise
            // the very first frame after activity launch sees a stale
            // sentinel and doesn't update until the next inset change).
            decorView.requestApplyInsets()
        } else {
            // Legacy path (API 24-29): use the deprecated
            // `systemWindowInset*` accessors for safe area, and a
            // global-layout listener with visible-display-frame diff
            // for the IME path. The legacy IME path catches show /
            // hide but not the per-frame animation steps.
            decorView.setOnApplyWindowInsetsListener { v, insets ->
                @Suppress("DEPRECATION")
                val topLogical = (insets.systemWindowInsetTop.toFloat() / density).toInt()
                @Suppress("DEPRECATION")
                val rightLogical = (insets.systemWindowInsetRight.toFloat() / density).toInt()
                @Suppress("DEPRECATION")
                val bottomLogical = (insets.systemWindowInsetBottom.toFloat() / density).toInt()
                @Suppress("DEPRECATION")
                val leftLogical = (insets.systemWindowInsetLeft.toFloat() / density).toInt()
                if (topLogical != lastDispatchedSafeAreaTopPx
                    || rightLogical != lastDispatchedSafeAreaRightPx
                    || bottomLogical != lastDispatchedSafeAreaBottomPx
                    || leftLogical != lastDispatchedSafeAreaLeftPx
                ) {
                    lastDispatchedSafeAreaTopPx = topLogical
                    lastDispatchedSafeAreaRightPx = rightLogical
                    lastDispatchedSafeAreaBottomPx = bottomLogical
                    lastDispatchedSafeAreaLeftPx = leftLogical
                    try {
                        nativeDispatchSafeArea(topLogical, rightLogical, bottomLogical, leftLogical)
                    } catch (e: UnsatisfiedLinkError) {
                        // see modern branch
                    }
                }
                v.onApplyWindowInsets(insets)
            }
            decorView.requestApplyInsets()

            val rect = android.graphics.Rect()
            decorView.viewTreeObserver.addOnGlobalLayoutListener {
                decorView.getWindowVisibleDisplayFrame(rect)
                val screenHeight = decorView.rootView.height
                val keyboardPx = (screenHeight - rect.bottom).coerceAtLeast(0)
                val logicalPx = (keyboardPx.toFloat() / density).toInt()
                if (logicalPx != lastDispatchedImeInsetPx) {
                    lastDispatchedImeInsetPx = logicalPx
                    try {
                        nativeDispatchKeyboardInset(logicalPx)
                    } catch (e: UnsatisfiedLinkError) {
                        // see modern branch
                    }
                }
            }
        }
    }

    /**
     * Register a native function handler
     *
     * @param namespace The namespace (e.g., "device", "haptics")
     * @param name The function name
     * @param handler Handler that receives JSON args and returns a result
     */
    fun register(namespace: String, name: String, handler: (JSONArray) -> Any?) {
        handlers.getOrPut(namespace) { mutableMapOf() }[name] = handler
    }

    /**
     * Convenience: Register a no-arg function returning String
     */
    fun registerString(namespace: String, name: String, handler: () -> String) {
        register(namespace, name) { handler() }
    }

    /**
     * Convenience: Register a no-arg void function
     */
    fun registerVoid(namespace: String, name: String, handler: () -> Unit) {
        register(namespace, name) { handler(); null }
    }

    /**
     * Called from JNI to execute a registered function
     *
     * @param namespace The namespace
     * @param name The function name
     * @param argsJson JSON-encoded arguments array
     * @return JSON-encoded result or error
     */
    @JvmStatic
    fun callNative(namespace: String, name: String, argsJson: String): String {
        return try {
            val nsHandlers = handlers[namespace]
                ?: return errorJson("NotRegistered", "Namespace '$namespace' not found")

            val handler = nsHandlers[name]
                ?: return errorJson("NotRegistered", "Function '$namespace.$name' not found")

            val args = JSONArray(argsJson)
            val result = handler(args)

            successJson(result)
        } catch (e: Exception) {
            errorJson("PlatformError", e.message ?: "Unknown error")
        }
    }

    /**
     * Register default handlers for common functionality
     */
    fun registerDefaults(context: Context) {
        init(context)
        val ctx = context.applicationContext

        // =====================================================================
        // Device namespace
        // =====================================================================

        registerString("device", "get_battery_level") {
            val bm = ctx.getSystemService<BatteryManager>()
            bm?.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)?.toString() ?: "0"
        }

        registerString("device", "get_model") {
            Build.MODEL
        }

        registerString("device", "get_os_version") {
            Build.VERSION.RELEASE
        }

        register("device", "is_low_power_mode") {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                val pm = ctx.getSystemService(Context.POWER_SERVICE) as? android.os.PowerManager
                pm?.isPowerSaveMode ?: false
            } else {
                false
            }
        }

        register("device", "has_notch") {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                // Check for display cutout
                // This requires a window, return false as default
                false
            } else {
                false
            }
        }

        registerString("device", "get_locale") {
            Locale.getDefault().toString()
        }

        registerString("device", "get_timezone") {
            TimeZone.getDefault().id
        }

        // =====================================================================
        // Haptics namespace
        // =====================================================================

        register("haptics", "vibrate") { args ->
            val durationMs = args.optLong(0, 100)
            vibrate(ctx, durationMs)
            null
        }

        register("haptics", "impact") { args ->
            val style = when {
                !args.isNull(0) && args.optString(0).isNotEmpty() -> {
                    when (args.optString(0, "medium").lowercase()) {
                        "light" -> 0
                        "heavy" -> 2
                        else -> 1
                    }
                }
                else -> args.optInt(0, 1)
            }
            val amplitude = when (style) {
                0 -> 50   // light
                2 -> 255  // heavy
                else -> 128 // medium
            }
            vibrateWithAmplitude(ctx, 10, amplitude)
            null
        }

        registerVoid("haptics", "selection") {
            vibrateWithAmplitude(ctx, 5, 50)
        }

        registerVoid("haptics", "success") {
            vibrateWithAmplitude(ctx, 30, 200)
        }

        registerVoid("haptics", "warning") {
            vibrateWithAmplitude(ctx, 50, 150)
        }

        registerVoid("haptics", "error") {
            vibrateWithAmplitude(ctx, 100, 255)
        }

        // =====================================================================
        // Clipboard namespace
        // =====================================================================

        register("clipboard", "copy") { args ->
            val text = args.optString(0, "")
            val clipboard = ctx.getSystemService<ClipboardManager>()
            clipboard?.setPrimaryClip(ClipData.newPlainText("Blinc", text))
            null
        }

        registerString("clipboard", "paste") {
            val clipboard = ctx.getSystemService<ClipboardManager>()
            clipboard?.primaryClip?.getItemAt(0)?.text?.toString() ?: ""
        }

        register("clipboard", "has_content") {
            val clipboard = ctx.getSystemService<ClipboardManager>()
            clipboard?.hasPrimaryClip() ?: false
        }

        registerVoid("clipboard", "clear") {
            val clipboard = ctx.getSystemService<ClipboardManager>()
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                clipboard?.clearPrimaryClip()
            }
        }

        // =====================================================================
        // Text-edit context menu namespace
        // =====================================================================
        //
        // Mirrors the iOS `edit_menu` namespace. Rust text-editable
        // widgets call into this from their double-tap handlers to
        // show a native Android contextual menu (Cut / Copy / Paste /
        // Select All) over the focused selection.
        //
        // Android's equivalent of iOS `UIMenuController` is
        // `ActionMode` started against the activity's content view.
        // The action callbacks are routed back into Rust by
        // synthesizing the same Cmd+key codes the desktop runner
        // uses for the corresponding shortcuts:
        //
        //   Cut        → key code 88 (Cmd+X)
        //   Copy       → key code 67 (Cmd+C)
        //   Paste      → key code 86 (Cmd+V)
        //   Select All → key code 65 (Cmd+A)
        //
        // Each Blinc text-editable widget already handles those
        // shortcut codes, so the menu plugs into the existing
        // copy/cut/paste paths once the dispatch is wired up.
        //
        // Bitmask layout matches `text_edit::edit_menu_actions`:
        //   bit 0 = CUT
        //   bit 1 = COPY
        //   bit 2 = PASTE
        //   bit 3 = SELECT_ALL

        register("edit_menu", "show") { args ->
            val anchorX = (args.optDouble(0, 0.0)).toFloat()
            val anchorY = (args.optDouble(1, 0.0)).toFloat()
            val selX = (args.optDouble(2, anchorX.toDouble())).toFloat()
            val selY = (args.optDouble(3, anchorY.toDouble())).toFloat()
            val selW = (args.optDouble(4, 0.0)).toFloat()
            val selH = (args.optDouble(5, 24.0)).toFloat()
            val actions = args.optInt(6, 0)
            val activity = currentActivity()
            activity?.runOnUiThread {
                BlincEditMenuHelper.show(activity, anchorX, anchorY, selX, selY, selW, selH, actions)
            }
            null
        }

        registerVoid("edit_menu", "hide") {
            val activity = currentActivity()
            activity?.runOnUiThread {
                BlincEditMenuHelper.hide()
            }
        }

        // =====================================================================
        // Keyboard namespace
        // =====================================================================

        register("keyboard", "show") { _ ->
            // The Application context cannot show the soft keyboard — we
            // need an Activity for the decor view + input method service.
            // The Activity is stashed by `init(context)` when called with
            // an Activity (see MainActivity.onCreate), kept as a
            // WeakReference so we don't leak it.
            val activity = currentActivity()
            activity?.runOnUiThread {
                val imm = activity.getSystemService(Context.INPUT_METHOD_SERVICE)
                    as? InputMethodManager
                val view = activity.window?.decorView
                if (view != null && imm != null) {
                    // NativeActivity's decor view is not focusable by default,
                    // so the IMM ignores `showSoftInput` against it. Force
                    // it focusable, take focus, then request the keyboard.
                    view.isFocusable = true
                    view.isFocusableInTouchMode = true
                    view.requestFocus()
                    imm.showSoftInput(view, InputMethodManager.SHOW_FORCED)
                }
            }
            null
        }

        register("keyboard", "hide") { _ ->
            val activity = currentActivity()
            activity?.runOnUiThread {
                val imm = activity.getSystemService(Context.INPUT_METHOD_SERVICE)
                    as? InputMethodManager
                val token = activity.window?.decorView?.windowToken
                imm?.hideSoftInputFromWindow(token, 0)
            }
            null
        }

        // =====================================================================
        // Camera namespace
        // =====================================================================

        register("camera", "preview_start") { args ->
            val width = args.optInt(0, 640)
            val height = args.optInt(1, 480)
            val fps = args.optInt(2, 30)
            val facing = args.optInt(3, 0) // 0=front, 1=back
            val streamId = args.optLong(4, 0)

            startCameraPreview(ctx, width, height, fps, facing, streamId)
            null
        }

        registerVoid("camera", "preview_stop") {
            stopCameraPreview()
        }

        // =====================================================================
        // Audio recording namespace
        // =====================================================================

        register("audio", "record_start") { args ->
            val sampleRate = args.optInt(0, 44100)
            val channels = args.optInt(1, 1)
            val streamId = args.optLong(2, 0)

            startAudioRecording(ctx, sampleRate, channels, streamId)
            null
        }

        registerVoid("audio", "record_stop") {
            stopAudioRecording()
        }

        // =====================================================================
        // App namespace
        // =====================================================================

        registerString("app", "get_version") {
            try {
                ctx.packageManager.getPackageInfo(ctx.packageName, 0).versionName ?: "1.0"
            } catch (e: Exception) {
                "1.0"
            }
        }

        registerString("app", "get_build_number") {
            try {
                val info = ctx.packageManager.getPackageInfo(ctx.packageName, 0)
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                    info.longVersionCode.toString()
                } else {
                    @Suppress("DEPRECATION")
                    info.versionCode.toString()
                }
            } catch (e: Exception) {
                "1"
            }
        }

        registerString("app", "get_bundle_id") {
            ctx.packageName
        }

        register("app", "open_url") { args ->
            val url = args.optString(0, "")
            try {
                val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
                intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                ctx.startActivity(intent)
                true
            } catch (e: Exception) {
                false
            }
        }

        register("app", "share_text") { args ->
            val text = args.optString(0, "")
            val intent = Intent(Intent.ACTION_SEND).apply {
                type = "text/plain"
                putExtra(Intent.EXTRA_TEXT, text)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            ctx.startActivity(Intent.createChooser(intent, "Share").apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            })
            null
        }

        // =====================================================================
        // Permissions namespace
        // =====================================================================

        register("permissions", "has_location") {
            val granted = sensorCollector.hasLocationPermission()
            permissionCapabilityPayload(
                permission = Manifest.permission.ACCESS_FINE_LOCATION,
                granted = granted,
            )
        }
        register("permissions", "has_location_always") {
            val granted = sensorCollector.hasLocationAlwaysPermission()
            permissionCapabilityPayload(
                permission = Manifest.permission.ACCESS_BACKGROUND_LOCATION,
                granted = granted,
            )
        }
        register("permissions", "has_motion") {
            val granted = sensorCollector.hasMotionPermission()
            permissionCapabilityPayload(
                permission = Manifest.permission.ACTIVITY_RECOGNITION,
                granted = granted,
            )
        }
        register("permissions", "has_camera") {
            val granted = sensorCollector.hasCameraPermission()
            permissionCapabilityPayload(
                permission = Manifest.permission.CAMERA,
                granted = granted,
            )
        }
        register("permissions", "has_microphone") {
            val granted = sensorCollector.hasMicrophonePermission()
            permissionCapabilityPayload(
                permission = Manifest.permission.RECORD_AUDIO,
                granted = granted,
            )
        }
        register("permissions", "has_photos") {
            val granted = sensorCollector.hasPhotosPermission()
            permissionCapabilityPayload(
                permission = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    Manifest.permission.READ_MEDIA_IMAGES
                } else {
                    Manifest.permission.READ_EXTERNAL_STORAGE
                },
                granted = granted,
            )
        }
        register("permissions", "has_notifications") {
            val granted = sensorCollector.hasNotificationsPermission()
            permissionCapabilityPayload(
                permission = Manifest.permission.POST_NOTIFICATIONS,
                granted = granted,
            )
        }
        register("permissions", "has_bluetooth_scan") {
            val granted = sensorCollector.hasBluetoothScanPermission()
            permissionCapabilityPayload(
                permission = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    Manifest.permission.BLUETOOTH_SCAN
                } else {
                    Manifest.permission.ACCESS_FINE_LOCATION
                },
                granted = granted,
            )
        }
        register("permissions", "has_bluetooth_connect") {
            val granted = sensorCollector.hasBluetoothConnectPermission()
            permissionCapabilityPayload(
                permission = Manifest.permission.BLUETOOTH_CONNECT,
                granted = granted,
            )
        }
        register("permissions", "request_location_when_in_use") {
            val previous = permissionCapabilityState(
                permission = Manifest.permission.ACCESS_FINE_LOCATION,
                granted = sensorCollector.hasLocationPermission(),
            )
            val granted = sensorCollector.requestLocationPermissionWhenInUse()
            permissionRequestPayload(
                permission = Manifest.permission.ACCESS_FINE_LOCATION,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_location_always") {
            val previous = permissionCapabilityState(
                permission = Manifest.permission.ACCESS_BACKGROUND_LOCATION,
                granted = sensorCollector.hasLocationAlwaysPermission(),
            )
            val granted = sensorCollector.requestLocationPermissionAlways()
            permissionRequestPayload(
                permission = Manifest.permission.ACCESS_BACKGROUND_LOCATION,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_motion") {
            val previous = permissionCapabilityState(
                permission = Manifest.permission.ACTIVITY_RECOGNITION,
                granted = sensorCollector.hasMotionPermission(),
            )
            val granted = sensorCollector.requestMotionPermission()
            permissionRequestPayload(
                permission = Manifest.permission.ACTIVITY_RECOGNITION,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_camera") {
            val previous = permissionCapabilityState(
                permission = Manifest.permission.CAMERA,
                granted = sensorCollector.hasCameraPermission(),
            )
            val granted = sensorCollector.requestCameraPermission()
            permissionRequestPayload(
                permission = Manifest.permission.CAMERA,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_microphone") {
            val previous = permissionCapabilityState(
                permission = Manifest.permission.RECORD_AUDIO,
                granted = sensorCollector.hasMicrophonePermission(),
            )
            val granted = sensorCollector.requestMicrophonePermission()
            permissionRequestPayload(
                permission = Manifest.permission.RECORD_AUDIO,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_photos") {
            val permission = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                Manifest.permission.READ_MEDIA_IMAGES
            } else {
                Manifest.permission.READ_EXTERNAL_STORAGE
            }
            val previous = permissionCapabilityState(
                permission = permission,
                granted = sensorCollector.hasPhotosPermission(),
            )
            val granted = sensorCollector.requestPhotosPermission()
            permissionRequestPayload(
                permission = permission,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_notifications") {
            val previous = permissionCapabilityState(
                permission = Manifest.permission.POST_NOTIFICATIONS,
                granted = sensorCollector.hasNotificationsPermission(),
            )
            val granted = sensorCollector.requestNotificationsPermission()
            permissionRequestPayload(
                permission = Manifest.permission.POST_NOTIFICATIONS,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_bluetooth_scan") {
            val permission = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                Manifest.permission.BLUETOOTH_SCAN
            } else {
                Manifest.permission.ACCESS_FINE_LOCATION
            }
            val previous = permissionCapabilityState(
                permission = permission,
                granted = sensorCollector.hasBluetoothScanPermission(),
            )
            val granted = sensorCollector.requestBluetoothScanPermission()
            permissionRequestPayload(
                permission = permission,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "request_bluetooth_connect") {
            val previous = permissionCapabilityState(
                permission = Manifest.permission.BLUETOOTH_CONNECT,
                granted = sensorCollector.hasBluetoothConnectPermission(),
            )
            val granted = sensorCollector.requestBluetoothConnectPermission()
            permissionRequestPayload(
                permission = Manifest.permission.BLUETOOTH_CONNECT,
                previous = previous,
                granted = granted,
            )
        }
        register("permissions", "open_settings") {
            openApplicationSettings()
        }

        // =====================================================================
        // BLE namespace
        // =====================================================================

        register("ble", "configure") { args ->
            bleCollector.configure(args.optString(0, "{}"))
        }

        register("ble", "start") { args ->
            val sessionId = args.optString(0, "")
            bleCollector.start(sessionId)
        }

        register("ble", "stop") { args ->
            val sessionId = args.optString(0, "")
            bleCollector.stop(sessionId)
        }

        registerString("ble", "status") {
            bleCollector.statusJson()
        }

        register("ble", "drain_results") { args ->
            bleCollector.drainResults(args.optInt(0, 64))
        }

        // =====================================================================
        // Sensor namespace (default stubs)
        // =====================================================================

        register("sensor", "configure") { args ->
            sensorCollector.configure(args.optString(0, "{}"))
        }

        register("sensor", "start") { args ->
            val sessionId = args.optString(0, "")
            sensorCollector.start(sessionId)
        }

        register("sensor", "stop") { args ->
            val sessionId = args.optString(0, "")
            sensorCollector.stop(sessionId)
        }

        registerString("sensor", "status") {
            sensorCollector.statusJson()
        }

        register("sensor", "drain_frames") { args ->
            sensorCollector.drainFrames(args.optInt(0, 64))
        }

        register("sensor", "peek_frames") { args ->
            sensorCollector.peekFrames(args.optInt(0, 32))
        }

        registerString("sensor", "supported_kinds") {
            sensorCollector.supportedKindsJson()
        }

        registerVoid("sensor", "clear_buffer") {
            sensorCollector.clearBuffer()
        }
    }

    // =========================================================================
    // Helper functions
    // =========================================================================

    private fun successJson(value: Any?): String {
        val obj = JSONObject()
        obj.put("success", true)
        when (value) {
            null -> obj.put("value", JSONObject.NULL)
            is String -> obj.put("value", value)
            is Boolean -> obj.put("value", value)
            is Int -> obj.put("value", value)
            is Long -> obj.put("value", value)
            is Float -> obj.put("value", value)
            is Double -> obj.put("value", value)
            is JSONObject -> obj.put("value", value)
            is JSONArray -> obj.put("value", value)
            is ByteArray -> obj.put("value", android.util.Base64.encodeToString(value, android.util.Base64.NO_WRAP))
            else -> obj.put("value", value.toString())
        }
        return obj.toString()
    }

    private fun errorJson(type: String, message: String): String {
        val obj = JSONObject()
        obj.put("success", false)
        obj.put("errorType", type)
        obj.put("errorMessage", message)
        return obj.toString()
    }

    private fun openApplicationSettings(): Boolean {
        val activity = foregroundActivityRef?.get() ?: return false
        val intent = Intent(android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
            data = Uri.fromParts("package", activity.packageName, null)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        activity.startActivity(intent)
        return true
    }

    private fun permissionCapabilityState(
        permission: String,
        granted: Boolean,
        supported: Boolean = true,
    ): PermissionCapabilityState {
        if (!supported) {
            return PermissionCapabilityState(
                status = "unknown",
                canRequest = false,
                requiresSettingsRedirect = false,
                supported = false,
            )
        }
        if (granted) {
            return PermissionCapabilityState(
                status = "granted",
                canRequest = false,
                requiresSettingsRedirect = false,
            )
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return PermissionCapabilityState(
                status = "granted",
                canRequest = false,
                requiresSettingsRedirect = false,
            )
        }

        val activity = foregroundActivityRef?.get()
        val requestedBefore = activity != null && activity.getPreferences(Context.MODE_PRIVATE)
            .getBoolean("permission_requested:$permission", false)
        val shouldShowRationale = activity?.shouldShowRequestPermissionRationale(permission) == true
        return when {
            requestedBefore && !shouldShowRationale -> PermissionCapabilityState(
                status = "permanently_denied",
                canRequest = false,
                requiresSettingsRedirect = true,
            )
            requestedBefore -> PermissionCapabilityState(
                status = "denied",
                canRequest = true,
                requiresSettingsRedirect = false,
            )
            else -> PermissionCapabilityState(
                status = "not_determined",
                canRequest = true,
                requiresSettingsRedirect = false,
            )
        }
    }

    private fun permissionCapabilityPayload(
        permission: String,
        granted: Boolean,
        supported: Boolean = true,
    ): JSONObject {
        val state = permissionCapabilityState(
            permission = permission,
            granted = granted,
            supported = supported,
        )
        return JSONObject()
            .put("status", state.status)
            .put("canRequest", state.canRequest)
            .put("requiresSettingsRedirect", state.requiresSettingsRedirect)
            .put("supported", state.supported)
    }

    private fun permissionRequestPayload(
        permission: String,
        previous: PermissionCapabilityState,
        granted: Boolean,
        supported: Boolean = true,
    ): JSONObject {
        val current = when {
            !supported -> permissionCapabilityState(permission, granted = false, supported = false)
            granted -> permissionCapabilityState(permission, granted = true, supported = true)
            previous.canRequest && !previous.requiresSettingsRedirect -> previous
            else -> permissionCapabilityState(permission, granted = false, supported = true)
        }
        return JSONObject()
            .put("status", current.status)
            .put("previousStatus", previous.status)
            .put("canRequestAgain", current.canRequest)
            .put("requiresSettingsRedirect", current.requiresSettingsRedirect)
    }

    private fun markPermissionRequested(permission: String) {
        val activity = foregroundActivityRef?.get() ?: return
        activity.getPreferences(Context.MODE_PRIVATE)
            .edit()
            .putBoolean("permission_requested:$permission", true)
            .apply()
    }

    private fun vibrate(context: Context, durationMs: Long) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            val vm = context.getSystemService<VibratorManager>()
            vm?.defaultVibrator?.vibrate(
                VibrationEffect.createOneShot(durationMs, VibrationEffect.DEFAULT_AMPLITUDE)
            )
        } else {
            @Suppress("DEPRECATION")
            val v = context.getSystemService<Vibrator>()
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                v?.vibrate(VibrationEffect.createOneShot(durationMs, VibrationEffect.DEFAULT_AMPLITUDE))
            } else {
                @Suppress("DEPRECATION")
                v?.vibrate(durationMs)
            }
        }
    }

    private fun vibrateWithAmplitude(context: Context, durationMs: Long, amplitude: Int) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val vm = context.getSystemService<VibratorManager>()
                vm?.defaultVibrator?.vibrate(VibrationEffect.createOneShot(durationMs, amplitude))
            } else {
                @Suppress("DEPRECATION")
                val v = context.getSystemService<Vibrator>()
                v?.vibrate(VibrationEffect.createOneShot(durationMs, amplitude))
            }
        } else {
            vibrate(context, durationMs)
        }
    }

    // =========================================================================
    // Camera preview
    // =========================================================================

    private var cameraStreamId: Long = 0
    private var isCameraRunning = false

    /**
     * Start camera preview and stream RGBA frames to Rust via JNI.
     *
     * Uses Camera2 API. Each frame is converted to RGBA and sent via
     * nativeDispatchStreamData(streamId, rgbaBytes).
     */
    private fun startCameraPreview(
        context: Context,
        width: Int, height: Int, fps: Int, facing: Int, streamId: Long
    ) {
        cameraStreamId = streamId
        isCameraRunning = true

        // Camera2 implementation requires android.hardware.camera2 imports
        // and a background HandlerThread. This is a template — users should
        // adapt to their specific camera requirements.
        //
        // The key integration point:
        // 1. Open CameraDevice for the requested facing
        // 2. Create ImageReader with ImageFormat.YUV_420_888
        // 3. In OnImageAvailableListener, convert YUV → RGBA
        // 4. Call: nativeDispatchStreamData(streamId, rgbaBytes)
        //
        // Example conversion (simplified):
        // val image = reader.acquireLatestImage()
        // val rgba = yuvToRgba(image)  // convert planes to RGBA
        // nativeDispatchStreamData(cameraStreamId, rgba)
        // image.close()

        android.util.Log.i("BlincNativeBridge", "Camera preview started: ${width}x${height} @ ${fps}fps, stream=$streamId")
    }

    private fun stopCameraPreview() {
        isCameraRunning = false
        android.util.Log.i("BlincNativeBridge", "Camera preview stopped")
    }

    // =========================================================================
    // Audio recording
    // =========================================================================

    private var audioStreamId: Long = 0
    private var isAudioRecording = false
    private var audioRecordThread: Thread? = null

    /**
     * Start audio recording and stream PCM samples to Rust.
     *
     * Uses AudioRecord API. PCM float samples are sent as raw bytes via
     * nativeDispatchStreamData(streamId, pcmBytes).
     */
    private fun startAudioRecording(
        context: Context,
        sampleRate: Int, channels: Int, streamId: Long
    ) {
        audioStreamId = streamId
        isAudioRecording = true

        val channelConfig = if (channels == 1)
            android.media.AudioFormat.CHANNEL_IN_MONO
        else
            android.media.AudioFormat.CHANNEL_IN_STEREO

        val bufferSize = android.media.AudioRecord.getMinBufferSize(
            sampleRate, channelConfig, android.media.AudioFormat.ENCODING_PCM_FLOAT
        )

        audioRecordThread = Thread {
            try {
                val recorder = android.media.AudioRecord(
                    android.media.MediaRecorder.AudioSource.MIC,
                    sampleRate, channelConfig,
                    android.media.AudioFormat.ENCODING_PCM_FLOAT,
                    bufferSize
                )
                recorder.startRecording()

                val buffer = FloatArray(bufferSize / 4)
                while (isAudioRecording) {
                    val read = recorder.read(buffer, 0, buffer.size, android.media.AudioRecord.READ_BLOCKING)
                    if (read > 0) {
                        // Convert float array to byte array (little-endian)
                        val bytes = ByteArray(read * 4)
                        val bb = java.nio.ByteBuffer.wrap(bytes).order(java.nio.ByteOrder.LITTLE_ENDIAN)
                        for (i in 0 until read) {
                            bb.putFloat(buffer[i])
                        }
                        nativeDispatchStreamData(audioStreamId, bytes)
                    }
                }

                recorder.stop()
                recorder.release()
            } catch (e: Exception) {
                android.util.Log.e("BlincNativeBridge", "Audio recording error: ${e.message}")
            }
        }
        audioRecordThread?.start()
    }

    private fun stopAudioRecording() {
        isAudioRecording = false
        audioRecordThread?.join(1000)
        audioRecordThread = null
    }

    // JNI bridge for stream data
    @JvmStatic
    external fun nativeDispatchStreamData(streamId: Long, data: ByteArray)

    // JNI bridge for soft-keyboard inset updates.
    //
    // Called from `attachWindowInsetsListener` whenever
    // `WindowInsets.Type.ime().bottom` changes. The Rust runtime
    // (`Java_com_blinc_BlincNativeBridge_nativeDispatchKeyboardInset` in
    // `crates/blinc_app/src/android.rs`) stores the value in a global
    // atomic that the `android_main` poll loop reads on every tick to
    // drive the "scroll focused text input above the keyboard" behavior.
    //
    // The Kotlin side already converts the raw physical-pixel value
    // from `WindowInsets` into LOGICAL pixels by dividing by the
    // display density, so the Rust side gets a value directly comparable
    // to `WindowedContext.height`.
    @JvmStatic
    external fun nativeDispatchKeyboardInset(insetLogicalPx: Int)

    // JNI bridge for system-bar safe-area inset updates.
    //
    // Called from `attachWindowInsetsListener` whenever the status bar,
    // navigation bar, notch cutout, or gesture bar inset changes. The
    // Rust runtime (`Java_com_blinc_BlincNativeBridge_nativeDispatchSafeArea`
    // in `crates/blinc_app/src/android.rs`) stores the four values in
    // global atomics; the `android_main` poll loop copies them into
    // `WindowedContext.safe_area` when any edge changes.
    //
    // All four values are logical pixels (already divided by display
    // density), matching `WindowedContext.width` / `height`. The tuple
    // order is `(top, right, bottom, left)` — the same order the blinc
    // `Window::safe_area_insets` trait method uses.
    @JvmStatic
    external fun nativeDispatchSafeArea(top: Int, right: Int, bottom: Int, left: Int)

    // JNI bridge for synthesized key-down events with modifier flags.
    //
    // Called from `BlincEditMenuHelper.onActionItemClicked` when the
    // user picks Cut / Copy / Paste / Select All from the native edit
    // menu. The Rust handler
    // (`Java_com_blinc_BlincNativeBridge_nativeDispatchKeyDownWithModifiers`
    // in `crates/blinc_app/src/android.rs`) queues the event and the
    // android_main poll loop dispatches it through `tree.broadcast_key_event`
    // on the next tick — which lands in the existing Cmd-shortcut
    // branch of every Blinc text-editable widget's `on_key_down`
    // handler.
    //
    // `modifiers` bitmask: shift=0x01, ctrl=0x02, alt=0x04, meta=0x08.
    // The edit menu callbacks always set the meta bit so the dispatch
    // routes through the Cmd-shortcut path.
    @JvmStatic
    external fun nativeDispatchKeyDownWithModifiers(keyCode: Int, modifiers: Int)
}

// =============================================================================
// Edit menu helper
// =============================================================================

/**
 * Native Android contextual edit menu (Cut / Copy / Paste / Select All)
 * shown over the focused Blinc text-editable widget on double-tap.
 *
 * Mirrors the iOS `BlincEditMenuHelper`. Uses
 * [android.view.ActionMode] (the framework-level equivalent of iOS's
 * UIMenuController) anchored at the position the Rust side passed in
 * via `edit_menu.show`.
 *
 * Action callbacks need to be wired through to Rust via the same
 * shortcut-key dispatch path Blinc's text-editable widgets already
 * use for Cmd+X / Cmd+C / Cmd+V / Cmd+A. That requires a JNI export
 * for `handleKeyDownWithModifiers` (or similar) which doesn't exist
 * yet — until then the menu shows on screen so the user gets visual
 * feedback that the gesture registered, and the action dispatch is
 * a TODO.
 */
object BlincEditMenuHelper {
    private var currentActionMode: android.view.ActionMode? = null

    fun show(
        activity: android.app.Activity,
        anchorX: Float,
        anchorY: Float,
        @Suppress("UNUSED_PARAMETER") selectionX: Float,
        @Suppress("UNUSED_PARAMETER") selectionY: Float,
        @Suppress("UNUSED_PARAMETER") selectionWidth: Float,
        @Suppress("UNUSED_PARAMETER") selectionHeight: Float,
        actions: Int,
    ) {
        // Dismiss any existing menu first.
        hide()

        val rootView = activity.window?.decorView?.rootView ?: return
        val callback = object : android.view.ActionMode.Callback {
            override fun onCreateActionMode(mode: android.view.ActionMode, menu: android.view.Menu): Boolean {
                if (actions and 0x01 != 0) {
                    menu.add(0, android.R.id.cut, 0, android.R.string.cut)
                }
                if (actions and 0x02 != 0) {
                    menu.add(0, android.R.id.copy, 1, android.R.string.copy)
                }
                if (actions and 0x04 != 0) {
                    menu.add(0, android.R.id.paste, 2, android.R.string.paste)
                }
                if (actions and 0x08 != 0) {
                    menu.add(0, android.R.id.selectAll, 3, android.R.string.selectAll)
                }
                return true
            }

            override fun onPrepareActionMode(mode: android.view.ActionMode, menu: android.view.Menu): Boolean = false

            override fun onActionItemClicked(mode: android.view.ActionMode, item: android.view.MenuItem): Boolean {
                // Dispatch the matching Cmd+key into Rust via the
                // JNI key-down-with-modifiers export. The Blinc
                // text-editable widgets already handle these
                // shortcut codes in their `on_key_down` handlers,
                // so the menu plugs into the same code path the
                // hardware-keyboard shortcuts use on every platform.
                //
                // We always set the meta (Cmd) modifier bit (0x08)
                // since the widget handlers gate clipboard ops
                // behind `ctx.meta`.
                val keyCode = when (item.itemId) {
                    android.R.id.cut -> 88        // Cmd+X
                    android.R.id.copy -> 67       // Cmd+C
                    android.R.id.paste -> 86      // Cmd+V
                    android.R.id.selectAll -> 65  // Cmd+A
                    else -> -1
                }
                if (keyCode >= 0) {
                    try {
                        // Fully qualified — this method lives on the
                        // top-level `BlincNativeBridge` object, not
                        // on `BlincEditMenuHelper`. Calling it
                        // unqualified from inside the helper trips
                        // an Unresolved reference at the Kotlin
                        // compile step.
                        BlincNativeBridge.nativeDispatchKeyDownWithModifiers(keyCode, 0x08)
                    } catch (e: UnsatisfiedLinkError) {
                        // Native side not loaded — silently ignore so
                        // the menu still dismisses cleanly.
                    }
                }
                mode.finish()
                return true
            }

            override fun onDestroyActionMode(mode: android.view.ActionMode) {
                if (currentActionMode === mode) {
                    currentActionMode = null
                }
            }
        }

        currentActionMode = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            // API 23+: use the floating action mode anchored to a
            // rect in the root view's coordinate space, which is
            // closer to the iOS UIMenuController behavior.
            rootView.startActionMode(
                object : android.view.ActionMode.Callback2() {
                    override fun onCreateActionMode(mode: android.view.ActionMode, menu: android.view.Menu) =
                        callback.onCreateActionMode(mode, menu)
                    override fun onPrepareActionMode(mode: android.view.ActionMode, menu: android.view.Menu) =
                        callback.onPrepareActionMode(mode, menu)
                    override fun onActionItemClicked(mode: android.view.ActionMode, item: android.view.MenuItem) =
                        callback.onActionItemClicked(mode, item)
                    override fun onDestroyActionMode(mode: android.view.ActionMode) =
                        callback.onDestroyActionMode(mode)
                    override fun onGetContentRect(
                        mode: android.view.ActionMode,
                        view: android.view.View,
                        outRect: android.graphics.Rect,
                    ) {
                        // Anchor the menu over the tap point. Convert
                        // logical pixels to physical pixels using the
                        // display density (the Rust side passes
                        // logical px / DIP).
                        val density = activity.resources.displayMetrics.density
                        val px = (anchorX * density).toInt()
                        val py = (anchorY * density).toInt()
                        outRect.set(px, py, px + 1, py + 1)
                    }
                },
                android.view.ActionMode.TYPE_FLOATING,
            )
        } else {
            // API < 23: legacy primary action mode (sticks to the top
            // of the screen). Less ideal but functional.
            rootView.startActionMode(callback)
        }
    }

    fun hide() {
        currentActionMode?.finish()
        currentActionMode = null
    }
}

private data class AndroidSensorConfig(
    val enabled: Set<String> = setOf("gps", "accelerometer", "gyroscope"),
    val gpsHz: Int = 1,
    val imuHz: Int = 50,
    val frameFlushMs: Int = 200,
)

private class AndroidSensorCollector {
    private val lock = Any()
    private val frameBuffer: ArrayDeque<JSONObject> = ArrayDeque()
    private val accuracyBySensor: MutableMap<Int, Int> = mutableMapOf()
    private val recentStepDetectorNs: ArrayDeque<Long> = ArrayDeque()
    private val frameCountsByKind: MutableMap<String, Long> = mutableMapOf()
    private val latestSampleByKind: MutableMap<String, String> = mutableMapOf()

    private var context: Context? = null
    private var foregroundActivityRef: WeakReference<Activity>? = null
    private var sensorManager: SensorManager? = null
    private var locationManager: LocationManager? = null
    private var config: AndroidSensorConfig = AndroidSensorConfig()
    private var running: Boolean = false
    private var activeSessionId: String? = null
    private var seq: Long = 0L
    private var totalFrameCount: Long = 0L
    private var lastStatsLogMs: Long = 0L
    private var locationListener: LocationListener? = null

    private val maxBufferedFrames = 4096
    private val cadenceWindowNs = 8_000_000_000L

    private val sensorListener = object : SensorEventListener {
        override fun onSensorChanged(event: SensorEvent) {
            val sensorType = event.sensor.type
            val monotonicNs = event.timestamp
            val accuracy = mapSensorAccuracy(event.accuracy)
            if (sensorType == Sensor.TYPE_STEP_DETECTOR) {
                handleStepDetectorEvent(monotonicNs, accuracy, event.values.copyOf())
                return
            }
            val kind = sensorTypeToKind(sensorType) ?: return
            appendFrame(kind, monotonicNs, accuracy, event.values.copyOf())
            if (kind == "rotation_vector" && synchronized(lock) { config.enabled.contains("quaternion") }) {
                appendFrame("quaternion", monotonicNs, accuracy, event.values.copyOf())
            }
        }

        override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) {
            if (sensor != null) {
                synchronized(lock) {
                    accuracyBySensor[sensor.type] = accuracy
                }
            }
        }
    }

    fun attach(appContext: Context) {
        context = appContext
        sensorManager = appContext.getSystemService(Context.SENSOR_SERVICE) as? SensorManager
        locationManager = appContext.getSystemService(Context.LOCATION_SERVICE) as? LocationManager
    }

    fun setForegroundActivity(activity: Activity?) {
        foregroundActivityRef = if (activity == null) null else WeakReference(activity)
    }

    private fun markPermissionRequested(permission: String) {
        val activity = foregroundActivityRef?.get() ?: return
        activity.getPreferences(Context.MODE_PRIVATE)
            .edit()
            .putBoolean("permission_requested:$permission", true)
            .apply()
    }

    fun configure(configJson: String): Boolean {
        return runCatching {
            val root = JSONObject(configJson)
            val enabled = mutableSetOf<String>()
            root.optJSONArray("enabled")?.let { arr ->
                for (i in 0 until arr.length()) {
                    val kind = arr.optString(i, "").trim()
                    if (kind.isNotEmpty()) {
                        enabled.add(kind)
                    }
                }
            }
            val next = AndroidSensorConfig(
                enabled = if (enabled.isEmpty()) AndroidSensorConfig().enabled else enabled,
                gpsHz = root.optInt("gps_hz", 1).coerceAtLeast(1),
                imuHz = root.optInt("imu_hz", 50).coerceAtLeast(1),
                frameFlushMs = root.optInt("frame_flush_ms", 200).coerceAtLeast(20),
            )
            synchronized(lock) {
                config = next
            }
            true
        }.getOrElse { false }
    }

    fun start(sessionId: String): Boolean {
        val normalizedId = sessionId.trim()
        if (normalizedId.isEmpty()) {
            return false
        }
        val ctx = context ?: return false
        val localConfig: AndroidSensorConfig
        var restarting = false

        synchronized(lock) {
            if (running) {
                restarting = true
                stopInternalLocked()
            }
            running = true
            activeSessionId = normalizedId
            resetFrameStatsLocked()
            localConfig = config
        }

        if (restarting) {
            stopSensors()
        }
        startSensors(ctx)
        Log.i(
            TAG,
            "start: session=$normalizedId enabled=${localConfig.enabled} gpsHz=${localConfig.gpsHz} imuHz=${localConfig.imuHz}",
        )
        return true
    }

    fun stop(sessionId: String): Boolean {
        var stoppedSession: String? = null
        synchronized(lock) {
            if (!running) {
                return true
            }
            if (sessionId.isNotBlank() && sessionId != activeSessionId) {
                return false
            }
            stoppedSession = activeSessionId
            stopInternalLocked()
        }
        stopSensors()
        Log.i(TAG, "stop: session=${stoppedSession ?: "-"}")
        return true
    }

    fun statusJson(): String {
        val payload = JSONObject()
        synchronized(lock) {
            payload.put("running", running)
            payload.put("buffered_frames", frameBuffer.size)
            if (activeSessionId == null) {
                payload.put("active_session_id", JSONObject.NULL)
            } else {
                payload.put("active_session_id", activeSessionId)
            }
        }
        return payload.toString()
    }

    fun drainFrames(maxFrames: Int): String {
        val count = maxFrames.coerceIn(1, 2048)
        val arr = JSONArray()
        synchronized(lock) {
            repeat(count) {
                if (frameBuffer.isEmpty()) {
                    return@repeat
                }
                arr.put(frameBuffer.removeFirst())
            }
        }
        return arr.toString()
    }

    fun peekFrames(maxFrames: Int): String {
        val count = maxFrames.coerceIn(1, 256)
        val arr = JSONArray()
        synchronized(lock) {
            val start = (frameBuffer.size - count).coerceAtLeast(0)
            var index = 0
            for (frame in frameBuffer) {
                if (index >= start) {
                    arr.put(frame)
                }
                index += 1
            }
        }
        return arr.toString()
    }

    fun clearBuffer() {
        synchronized(lock) {
            frameBuffer.clear()
        }
    }

    fun supportedKindsJson(): String {
        if (context == null) {
            return "[]"
        }
        val sm = sensorManager ?: return "[]"
        val lm = locationManager
        val kinds = mutableListOf<String>()

        if (lm != null && lm.allProviders?.isNotEmpty() == true) {
            kinds.add("gps")
        }
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_ACCELEROMETER, "accelerometer")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_LINEAR_ACCELERATION, "linear_acceleration")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_GRAVITY, "gravity")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_GYROSCOPE, "gyroscope")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_ROTATION_VECTOR, "rotation_vector")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_ROTATION_VECTOR, "quaternion")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_MAGNETIC_FIELD, "magnetometer")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_PRESSURE, "barometer")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_LIGHT, "ambient_light")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_PROXIMITY, "proximity")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_AMBIENT_TEMPERATURE, "ambient_temperature")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_RELATIVE_HUMIDITY, "relative_humidity")
        addIfSensorAvailable(kinds, sm, Sensor.TYPE_STEP_COUNTER, "step_counter")
        val hasStepDetector = sm.getDefaultSensor(Sensor.TYPE_STEP_DETECTOR) != null
        if (hasStepDetector) {
            kinds.add("step_detector")
            kinds.add("cadence")
            kinds.add("activity")
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT_WATCH) {
            addIfSensorAvailable(kinds, sm, Sensor.TYPE_HEART_RATE, "heart_rate")
        }

        return JSONArray(kinds.distinct()).toString()
    }

    fun hasLocationPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        val fine = ctx.checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED
        val coarse = ctx.checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED
        return fine || coarse
    }

    fun hasLocationAlwaysPermission(): Boolean {
        if (!hasLocationPermission()) {
            return false
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return true
        }
        val ctx = context ?: return false
        return ctx.checkSelfPermission(Manifest.permission.ACCESS_BACKGROUND_LOCATION) ==
            PackageManager.PERMISSION_GRANTED
    }

    fun hasMotionPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return true
        }
        return ctx.checkSelfPermission(Manifest.permission.ACTIVITY_RECOGNITION) ==
            PackageManager.PERMISSION_GRANTED
    }

    fun hasCameraPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        return ctx.checkSelfPermission(Manifest.permission.CAMERA) ==
            PackageManager.PERMISSION_GRANTED
    }

    fun hasMicrophonePermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        return ctx.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED
    }

    fun hasPhotosPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            ctx.checkSelfPermission(Manifest.permission.READ_MEDIA_IMAGES) ==
                PackageManager.PERMISSION_GRANTED
        } else {
            @Suppress("DEPRECATION")
            ctx.checkSelfPermission(Manifest.permission.READ_EXTERNAL_STORAGE) ==
                PackageManager.PERMISSION_GRANTED
        }
    }

    fun hasNotificationsPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            return true
        }
        return ctx.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) ==
            PackageManager.PERMISSION_GRANTED
    }

    fun hasBluetoothScanPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            return ctx.checkSelfPermission(Manifest.permission.BLUETOOTH_SCAN) ==
                PackageManager.PERMISSION_GRANTED
        }
        return hasLocationPermission()
    }

    fun hasBluetoothConnectPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            return ctx.checkSelfPermission(Manifest.permission.BLUETOOTH_CONNECT) ==
                PackageManager.PERMISSION_GRANTED
        }
        return true
    }

    fun requestLocationPermissionWhenInUse(): Boolean {
        if (hasLocationPermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        val activity = foregroundActivityRef?.get() ?: return false
        val missing = mutableListOf<String>()
        if (activity.checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) !=
            PackageManager.PERMISSION_GRANTED
        ) {
            missing.add(Manifest.permission.ACCESS_FINE_LOCATION)
        }
        if (activity.checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) !=
            PackageManager.PERMISSION_GRANTED
        ) {
            missing.add(Manifest.permission.ACCESS_COARSE_LOCATION)
        }
        if (missing.isEmpty()) {
            return true
        }
        missing.forEach(::markPermissionRequested)
        activity.runOnUiThread {
            activity.requestPermissions(missing.toTypedArray(), REQUEST_CODE_LOCATION_WHEN_IN_USE)
        }
        return false
    }

    fun requestLocationPermissionAlways(): Boolean {
        val whenInUseGranted = requestLocationPermissionWhenInUse()
        if (!whenInUseGranted) {
            return false
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return true
        }
        val activity = foregroundActivityRef?.get() ?: return false
        val granted = activity.checkSelfPermission(Manifest.permission.ACCESS_BACKGROUND_LOCATION) ==
            PackageManager.PERMISSION_GRANTED
        if (granted) {
            return true
        }
        markPermissionRequested(Manifest.permission.ACCESS_BACKGROUND_LOCATION)
        activity.runOnUiThread {
            activity.requestPermissions(
                arrayOf(Manifest.permission.ACCESS_BACKGROUND_LOCATION),
                REQUEST_CODE_LOCATION_ALWAYS,
            )
        }
        return false
    }

    fun requestMotionPermission(): Boolean {
        if (hasMotionPermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return true
        }
        markPermissionRequested(Manifest.permission.ACTIVITY_RECOGNITION)
        requestPermissions(arrayOf(Manifest.permission.ACTIVITY_RECOGNITION), REQUEST_CODE_MOTION)
        return false
    }

    fun requestCameraPermission(): Boolean {
        if (hasCameraPermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        markPermissionRequested(Manifest.permission.CAMERA)
        requestPermissions(arrayOf(Manifest.permission.CAMERA), REQUEST_CODE_CAMERA)
        return false
    }

    fun requestMicrophonePermission(): Boolean {
        if (hasMicrophonePermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        markPermissionRequested(Manifest.permission.RECORD_AUDIO)
        requestPermissions(arrayOf(Manifest.permission.RECORD_AUDIO), REQUEST_CODE_MICROPHONE)
        return false
    }

    fun requestPhotosPermission(): Boolean {
        if (hasPhotosPermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            return true
        }
        val permissions = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            arrayOf(Manifest.permission.READ_MEDIA_IMAGES)
        } else {
            @Suppress("DEPRECATION")
            arrayOf(Manifest.permission.READ_EXTERNAL_STORAGE)
        }
        permissions.forEach(::markPermissionRequested)
        requestPermissions(permissions, REQUEST_CODE_PHOTOS)
        return false
    }

    fun requestNotificationsPermission(): Boolean {
        if (hasNotificationsPermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            return true
        }
        markPermissionRequested(Manifest.permission.POST_NOTIFICATIONS)
        requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), REQUEST_CODE_NOTIFICATIONS)
        return false
    }

    fun requestBluetoothScanPermission(): Boolean {
        if (hasBluetoothScanPermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            markPermissionRequested(Manifest.permission.BLUETOOTH_SCAN)
            requestPermissions(arrayOf(Manifest.permission.BLUETOOTH_SCAN), REQUEST_CODE_BLUETOOTH_SCAN)
            return false
        }
        return requestLocationPermissionWhenInUse()
    }

    fun requestBluetoothConnectPermission(): Boolean {
        if (hasBluetoothConnectPermission()) {
            return true
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) {
            return true
        }
        markPermissionRequested(Manifest.permission.BLUETOOTH_CONNECT)
        requestPermissions(
            arrayOf(Manifest.permission.BLUETOOTH_CONNECT),
            REQUEST_CODE_BLUETOOTH_CONNECT,
        )
        return false
    }

    private fun requestPermissions(permissions: Array<String>, requestCode: Int) {
        val activity = foregroundActivityRef?.get() ?: return
        activity.runOnUiThread {
            activity.requestPermissions(permissions, requestCode)
        }
    }

    companion object {
        private const val TAG = "BlincSensor"
        private const val STATS_LOG_INTERVAL_MS = 1_000L
        private const val REQUEST_CODE_LOCATION_WHEN_IN_USE = 3101
        private const val REQUEST_CODE_LOCATION_ALWAYS = 3102
        private const val REQUEST_CODE_MOTION = 3103
        private const val REQUEST_CODE_CAMERA = 3104
        private const val REQUEST_CODE_MICROPHONE = 3105
        private const val REQUEST_CODE_PHOTOS = 3106
        private const val REQUEST_CODE_NOTIFICATIONS = 3107
        private const val REQUEST_CODE_BLUETOOTH_SCAN = 3108
        private const val REQUEST_CODE_BLUETOOTH_CONNECT = 3109
    }

    private fun addIfSensorAvailable(
        out: MutableList<String>,
        sensorManager: SensorManager,
        sensorType: Int,
        kind: String,
    ) {
        if (sensorManager.getDefaultSensor(sensorType) != null) {
            out.add(kind)
        }
    }

    private fun startSensors(ctx: Context) {
        val localConfig = synchronized(lock) { config }
        val sm = sensorManager ?: return
        val imuDelayUs = (1_000_000L / localConfig.imuHz.toLong()).toInt().coerceAtLeast(2_000)

        fun registerSensor(sensorType: Int) {
            val sensor = sm.getDefaultSensor(sensorType) ?: return
            sm.registerListener(sensorListener, sensor, imuDelayUs)
        }

        if (localConfig.enabled.contains("accelerometer")) {
            registerSensor(Sensor.TYPE_ACCELEROMETER)
        }
        if (localConfig.enabled.contains("linear_acceleration")) {
            registerSensor(Sensor.TYPE_LINEAR_ACCELERATION)
        }
        if (localConfig.enabled.contains("gravity")) {
            registerSensor(Sensor.TYPE_GRAVITY)
        }
        if (localConfig.enabled.contains("gyroscope")) {
            registerSensor(Sensor.TYPE_GYROSCOPE)
        }
        if (localConfig.enabled.contains("rotation_vector") || localConfig.enabled.contains("quaternion")) {
            registerSensor(Sensor.TYPE_ROTATION_VECTOR)
        }
        if (localConfig.enabled.contains("magnetometer")) {
            registerSensor(Sensor.TYPE_MAGNETIC_FIELD)
        }
        if (localConfig.enabled.contains("barometer")) {
            registerSensor(Sensor.TYPE_PRESSURE)
        }
        if (localConfig.enabled.contains("ambient_light")) {
            registerSensor(Sensor.TYPE_LIGHT)
        }
        if (localConfig.enabled.contains("proximity")) {
            registerSensor(Sensor.TYPE_PROXIMITY)
        }
        if (localConfig.enabled.contains("ambient_temperature")) {
            registerSensor(Sensor.TYPE_AMBIENT_TEMPERATURE)
        }
        if (localConfig.enabled.contains("relative_humidity")) {
            registerSensor(Sensor.TYPE_RELATIVE_HUMIDITY)
        }
        if (localConfig.enabled.contains("step_counter")) {
            registerSensor(Sensor.TYPE_STEP_COUNTER)
        }
        if (localConfig.enabled.contains("step_detector") ||
            localConfig.enabled.contains("cadence") ||
            localConfig.enabled.contains("activity")
        ) {
            registerSensor(Sensor.TYPE_STEP_DETECTOR)
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT_WATCH &&
            localConfig.enabled.contains("heart_rate")
        ) {
            registerSensor(Sensor.TYPE_HEART_RATE)
        }

        if (localConfig.enabled.contains("gps")) {
            startLocationUpdates(ctx, localConfig.gpsHz)
        }
        Log.d(TAG, "registered sensors: enabled=${localConfig.enabled}")
    }

    @SuppressLint("MissingPermission")
    private fun startLocationUpdates(ctx: Context, gpsHz: Int) {
        if (!hasLocationPermission()) {
            return
        }
        val lm = locationManager ?: return
        val intervalMs = (1_000L / gpsHz.coerceAtLeast(1)).coerceAtLeast(250L)
        val provider = when {
            lm.isProviderEnabled(LocationManager.GPS_PROVIDER) -> LocationManager.GPS_PROVIDER
            lm.isProviderEnabled(LocationManager.NETWORK_PROVIDER) -> LocationManager.NETWORK_PROVIDER
            else -> null
        } ?: return

        locationListener = LocationListener { location ->
            appendLocationFrame(location)
        }

        lm.requestLocationUpdates(
            provider,
            intervalMs,
            0f,
            locationListener!!,
            Looper.getMainLooper(),
        )

        lm.getLastKnownLocation(provider)?.let { appendLocationFrame(it) }
    }

    private fun stopSensors() {
        sensorManager?.unregisterListener(sensorListener)
        locationListener?.let { listener ->
            locationManager?.removeUpdates(listener)
        }
        locationListener = null
    }

    private fun stopInternalLocked() {
        running = false
        activeSessionId = null
        accuracyBySensor.clear()
        recentStepDetectorNs.clear()
        resetFrameStatsLocked()
    }

    private fun handleStepDetectorEvent(monotonicNs: Long, accuracy: String, rawValues: FloatArray) {
        val (emitStepDetector, emitCadence, emitActivity, cadenceHz) = synchronized(lock) {
            val enabled = config.enabled
            val emitStepDetectorLocal = enabled.contains("step_detector")
            val emitCadenceLocal = enabled.contains("cadence")
            val emitActivityLocal = enabled.contains("activity")

            var cadenceHzLocal = 0f
            if (emitCadenceLocal || emitActivityLocal) {
                recentStepDetectorNs.addLast(monotonicNs)
                while (recentStepDetectorNs.isNotEmpty() &&
                    monotonicNs - recentStepDetectorNs.first() > cadenceWindowNs
                ) {
                    recentStepDetectorNs.removeFirst()
                }

                if (recentStepDetectorNs.size >= 2) {
                    val windowDurationNs = (monotonicNs - recentStepDetectorNs.first()).coerceAtLeast(1L)
                    val windowSeconds = windowDurationNs.toDouble() / 1_000_000_000.0
                    cadenceHzLocal = ((recentStepDetectorNs.size - 1).toDouble() / windowSeconds).toFloat()
                }
            }

            Quadruple(emitStepDetectorLocal, emitCadenceLocal, emitActivityLocal, cadenceHzLocal)
        }

        if (emitStepDetector) {
            appendFrame("step_detector", monotonicNs, accuracy, rawValues)
        }
        if (emitCadence) {
            appendFrame("cadence", monotonicNs, accuracy, floatArrayOf(cadenceHz))
        }
        if (emitActivity) {
            val activityValues = when {
                cadenceHz >= 2.0f -> floatArrayOf(0f, 0f, 1f, 0f, 0f, 0f) // running
                cadenceHz >= 0.5f -> floatArrayOf(0f, 1f, 0f, 0f, 0f, 0f) // walking
                else -> floatArrayOf(1f, 0f, 0f, 0f, 0f, 0f) // stationary
            }
            appendFrame("activity", monotonicNs, accuracy, activityValues)
        }
    }

    private fun appendLocationFrame(location: Location) {
        val monotonicNs = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.JELLY_BEAN_MR1) {
            location.elapsedRealtimeNanos
        } else {
            SystemClock.elapsedRealtimeNanos()
        }
        val unixTimeMs = location.time
        val values = floatArrayOf(
            location.latitude.toFloat(),
            location.longitude.toFloat(),
            location.altitude.toFloat(),
            location.speed,
            location.bearing,
            location.accuracy,
        )
        appendFrame(
            kind = "gps",
            monotonicNs = monotonicNs,
            accuracy = mapLocationAccuracy(location.accuracy),
            values = values,
            unixTimeMs = unixTimeMs,
        )
    }

    private fun appendFrame(
        kind: String,
        monotonicNs: Long,
        accuracy: String,
        values: FloatArray,
        unixTimeMs: Long? = null,
    ) {
        var statsLog: String? = null
        synchronized(lock) {
            if (!running) {
                return
            }
            val frame = JSONObject()
            frame.put("seq", ++seq)
            frame.put("sensor", kind)
            frame.put("time_monotonic_ns", monotonicNs)
            frame.put("time_unix_ms", unixTimeMs ?: monotonicToUnixMs(monotonicNs))
            frame.put("accuracy", accuracy)
            val valueArray = JSONArray()
            values.forEach { valueArray.put(it.toDouble()) }
            frame.put("values", valueArray)

            if (frameBuffer.size >= maxBufferedFrames) {
                frameBuffer.removeFirst()
            }
            frameBuffer.addLast(frame)

            totalFrameCount += 1
            frameCountsByKind[kind] = (frameCountsByKind[kind] ?: 0L) + 1L
            if (kind != "gps") {
                latestSampleByKind[kind] = formatValues(values)
            }

            val nowMs = SystemClock.elapsedRealtime()
            if (nowMs - lastStatsLogMs >= STATS_LOG_INTERVAL_MS) {
                lastStatsLogMs = nowMs
                val counts = frameCountsByKind
                    .entries
                    .sortedBy { it.key }
                    .joinToString(", ") { "${it.key}=${it.value}" }
                val latest = listOf("accelerometer", "gyroscope", "magnetometer", "barometer")
                    .mapNotNull { key ->
                        latestSampleByKind[key]?.let { sample -> "$key=[$sample]" }
                    }
                    .joinToString(" ")
                val session = activeSessionId ?: "-"
                statsLog = "session=$session buffered=${frameBuffer.size} total=$totalFrameCount kinds={$counts} latest=$latest"
            }
        }
        if (statsLog != null) {
            Log.d(TAG, "frame-stats ${statsLog}")
        }
    }

    private fun formatValues(values: FloatArray, maxElements: Int = 3): String {
        return values
            .take(maxElements)
            .joinToString(",") { value -> String.format(Locale.US, "%.3f", value) }
    }

    private fun resetFrameStatsLocked() {
        totalFrameCount = 0L
        lastStatsLogMs = 0L
        frameCountsByKind.clear()
        latestSampleByKind.clear()
    }

    private fun monotonicToUnixMs(monotonicNs: Long): Long {
        val nowMonoNs = SystemClock.elapsedRealtimeNanos()
        val nowUnixMs = System.currentTimeMillis()
        val deltaMs = ((nowMonoNs - monotonicNs).coerceAtLeast(0L)) / 1_000_000L
        return nowUnixMs - deltaMs
    }

    private fun mapSensorAccuracy(accuracy: Int): String {
        return when (accuracy) {
            SensorManager.SENSOR_STATUS_UNRELIABLE -> "unreliable"
            SensorManager.SENSOR_STATUS_ACCURACY_LOW -> "low"
            SensorManager.SENSOR_STATUS_ACCURACY_MEDIUM -> "medium"
            SensorManager.SENSOR_STATUS_ACCURACY_HIGH -> "high"
            else -> "medium"
        }
    }

    private fun mapLocationAccuracy(horizontalAccuracyMeters: Float): String {
        return when {
            horizontalAccuracyMeters <= 10f -> "high"
            horizontalAccuracyMeters <= 50f -> "medium"
            else -> "low"
        }
    }

    private fun sensorTypeToKind(sensorType: Int): String? {
        return when (sensorType) {
            Sensor.TYPE_ACCELEROMETER -> "accelerometer"
            Sensor.TYPE_LINEAR_ACCELERATION -> "linear_acceleration"
            Sensor.TYPE_GRAVITY -> "gravity"
            Sensor.TYPE_GYROSCOPE -> "gyroscope"
            Sensor.TYPE_ROTATION_VECTOR -> "rotation_vector"
            Sensor.TYPE_MAGNETIC_FIELD -> "magnetometer"
            Sensor.TYPE_PRESSURE -> "barometer"
            Sensor.TYPE_LIGHT -> "ambient_light"
            Sensor.TYPE_PROXIMITY -> "proximity"
            Sensor.TYPE_AMBIENT_TEMPERATURE -> "ambient_temperature"
            Sensor.TYPE_RELATIVE_HUMIDITY -> "relative_humidity"
            Sensor.TYPE_STEP_COUNTER -> "step_counter"
            Sensor.TYPE_STEP_DETECTOR -> "step_detector"
            Sensor.TYPE_HEART_RATE -> "heart_rate"
            else -> null
        }
    }
}

private data class AndroidBleScanConfig(
    val serviceUuids: List<String> = emptyList(),
    val allowDuplicates: Boolean = false,
    val scanMode: String? = null,
    val frameFlushMs: Int = 500,
)

private class AndroidBleCollector(
    private val permissions: AndroidSensorCollector,
) {
    private val lock = Any()
    private val resultBuffer: ArrayDeque<JSONObject> = ArrayDeque()

    private var context: Context? = null
    private var foregroundActivityRef: WeakReference<Activity>? = null
    private var bluetoothManager: BluetoothManager? = null
    private var running: Boolean = false
    private var activeSessionId: String? = null
    private var seq: Long = 0L
    private var config: AndroidBleScanConfig = AndroidBleScanConfig()

    private val maxBufferedResults = 4096

    private val scanCallback = object : ScanCallback() {
        override fun onScanResult(callbackType: Int, result: ScanResult?) {
            if (result != null) {
                appendResult(result)
            }
        }

        override fun onBatchScanResults(results: MutableList<ScanResult>?) {
            results?.forEach { appendResult(it) }
        }

        override fun onScanFailed(errorCode: Int) {
            Log.w(TAG, "BLE scan failed with error code=$errorCode")
        }
    }

    fun attach(appContext: Context) {
        context = appContext
        bluetoothManager = appContext.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
    }

    fun setForegroundActivity(activity: Activity?) {
        foregroundActivityRef = if (activity == null) null else WeakReference(activity)
    }

    private fun markPermissionRequested(permission: String) {
        val activity = foregroundActivityRef?.get() ?: return
        activity.getPreferences(Context.MODE_PRIVATE)
            .edit()
            .putBoolean("permission_requested:$permission", true)
            .apply()
    }

    fun configure(configJson: String): Boolean {
        return runCatching {
            val root = JSONObject(configJson)
            val serviceUuids = mutableListOf<String>()
            root.optJSONArray("service_uuids")?.let { arr ->
                for (i in 0 until arr.length()) {
                    val raw = arr.optString(i, "").trim()
                    if (raw.isNotEmpty()) {
                        serviceUuids.add(raw)
                    }
                }
            }
            val next = AndroidBleScanConfig(
                serviceUuids = serviceUuids,
                allowDuplicates = root.optBoolean("allow_duplicates", false),
                scanMode = root.optString("scan_mode", "").trim().ifEmpty { null },
                frameFlushMs = root.optInt("frame_flush_ms", 500).coerceAtLeast(0),
            )
            synchronized(lock) {
                config = next
            }
            true
        }.getOrDefault(false)
    }

    @SuppressLint("MissingPermission")
    fun start(sessionId: String): Boolean {
        val normalizedId = sessionId.trim()
        if (normalizedId.isEmpty()) {
            return false
        }

        if (!permissions.hasBluetoothScanPermission()) {
            return false
        }

        val adapter = bluetoothManager?.adapter ?: return false
        if (!adapter.isEnabled) {
            return false
        }

        val scanner = adapter.bluetoothLeScanner ?: return false
        var restarting = false
        val localConfig = synchronized(lock) {
            if (running) {
                restarting = true
                stopInternalLocked()
            }
            running = true
            activeSessionId = normalizedId
            seq = 0L
            resultBuffer.clear()
            config
        }
        if (restarting) {
            runCatching { scanner.stopScan(scanCallback) }
        }

        return try {
            val filters = localConfig.serviceUuids.mapNotNull { uuidText ->
                runCatching {
                    ScanFilter.Builder()
                        .setServiceUuid(ParcelUuid.fromString(uuidText))
                        .build()
                }.getOrNull()
            }
            val settings = ScanSettings.Builder()
                .setScanMode(scanModeFromConfig(localConfig.scanMode))
                .setReportDelay(localConfig.frameFlushMs.toLong())
                .build()
            scanner.startScan(filters, settings, scanCallback)
            Log.i(TAG, "start: session=$normalizedId filters=${filters.size}")
            true
        } catch (security: SecurityException) {
            synchronized(lock) { stopInternalLocked() }
            Log.w(TAG, "BLE start failed due to missing permission", security)
            false
        } catch (e: Exception) {
            synchronized(lock) { stopInternalLocked() }
            Log.w(TAG, "BLE start failed", e)
            false
        }
    }

    @SuppressLint("MissingPermission")
    fun stop(sessionId: String): Boolean {
        val adapter = bluetoothManager?.adapter
        val scanner = adapter?.bluetoothLeScanner
        synchronized(lock) {
            if (!running) {
                return true
            }
            if (sessionId.isNotBlank() && sessionId != activeSessionId) {
                return false
            }
            stopInternalLocked()
        }
        return try {
            scanner?.stopScan(scanCallback)
            true
        } catch (security: SecurityException) {
            Log.w(TAG, "BLE stop failed due to missing permission", security)
            false
        }
    }

    fun statusJson(): String {
        val payload = JSONObject()
        synchronized(lock) {
            payload.put("running", running)
            payload.put("buffered_results", resultBuffer.size)
            if (activeSessionId == null) {
                payload.put("active_session_id", JSONObject.NULL)
            } else {
                payload.put("active_session_id", activeSessionId)
            }
        }
        return payload.toString()
    }

    fun drainResults(maxResults: Int): String {
        val count = maxResults.coerceIn(1, 2048)
        val arr = JSONArray()
        synchronized(lock) {
            repeat(count) {
                if (resultBuffer.isEmpty()) {
                    return@repeat
                }
                arr.put(resultBuffer.removeFirst())
            }
        }
        return arr.toString()
    }

    private fun stopInternalLocked() {
        running = false
        activeSessionId = null
    }

    private fun scanModeFromConfig(raw: String?): Int {
        return when (raw?.trim()?.lowercase(Locale.US)) {
            "low_latency", "low-latency", "fast" -> ScanSettings.SCAN_MODE_LOW_LATENCY
            "balanced" -> ScanSettings.SCAN_MODE_BALANCED
            "opportunistic" -> ScanSettings.SCAN_MODE_OPPORTUNISTIC
            else -> ScanSettings.SCAN_MODE_LOW_POWER
        }
    }

    @SuppressLint("MissingPermission")
    private fun appendResult(result: ScanResult) {
        val nowMonoNs = SystemClock.elapsedRealtimeNanos()
        val monotonicNs = result.timestampNanos.takeIf { it > 0 } ?: nowMonoNs
        val deltaMs = ((nowMonoNs - monotonicNs).coerceAtLeast(0L)) / 1_000_000L
        val unixTimeMs = System.currentTimeMillis() - deltaMs
        val scanRecord = result.scanRecord

        val serviceUuids = JSONArray()
        scanRecord?.serviceUuids?.forEach { serviceUuids.put(it.uuid.toString()) }

        val txPower = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            result.txPower.takeUnless { it == Int.MAX_VALUE || it == 127 }
        } else {
            null
        }
        val connectable = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            result.isConnectable
        } else {
            null
        }
        val name = runCatching { scanRecord?.deviceName ?: result.device?.name }.getOrNull()
        val address = runCatching { result.device.address }.getOrDefault("")

        val payload = JSONObject()
        synchronized(lock) {
            if (!running) {
                return
            }
            seq += 1
            payload.put("seq", seq)
            payload.put("address", address)
            if (name.isNullOrBlank()) {
                payload.put("name", JSONObject.NULL)
            } else {
                payload.put("name", name)
            }
            payload.put("rssi", result.rssi)
            if (txPower == null) {
                payload.put("tx_power", JSONObject.NULL)
            } else {
                payload.put("tx_power", txPower)
            }
            if (connectable == null) {
                payload.put("is_connectable", JSONObject.NULL)
            } else {
                payload.put("is_connectable", connectable)
            }
            payload.put("service_uuids", serviceUuids)
            putOptionalString(payload, "manufacturer_data", encodeManufacturerData(scanRecord))
            putOptionalString(payload, "service_data", encodeServiceData(scanRecord))
            payload.put("time_monotonic_ns", monotonicNs)
            payload.put("time_unix_ms", unixTimeMs)

            if (resultBuffer.size >= maxBufferedResults) {
                resultBuffer.removeFirst()
            }
            resultBuffer.addLast(payload)
        }
    }

    private fun encodeManufacturerData(scanRecord: android.bluetooth.le.ScanRecord?): String? {
        val sparse = scanRecord?.manufacturerSpecificData ?: return null
        if (sparse.size() == 0) {
            return null
        }
        val key = sparse.keyAt(0)
        val value = sparse.valueAt(0) ?: return null
        val encoded = android.util.Base64.encodeToString(value, android.util.Base64.NO_WRAP)
        return "$key:$encoded"
    }

    private fun encodeServiceData(scanRecord: android.bluetooth.le.ScanRecord?): String? {
        val data = scanRecord?.serviceData ?: return null
        if (data.isEmpty()) {
            return null
        }
        val first = data.entries.firstOrNull() ?: return null
        val encoded = android.util.Base64.encodeToString(first.value, android.util.Base64.NO_WRAP)
        return "${first.key.uuid}:$encoded"
    }

    private fun putOptionalString(payload: JSONObject, key: String, value: String?) {
        if (value == null) {
            payload.put(key, JSONObject.NULL)
        } else {
            payload.put(key, value)
        }
    }

    companion object {
        private const val TAG = "BlincBle"
    }
}

private data class Quadruple<A, B, C, D>(
    val first: A,
    val second: B,
    val third: C,
    val fourth: D,
)
