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
import android.os.SystemClock
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.util.Log
import androidx.core.content.getSystemService
import org.json.JSONArray
import org.json.JSONObject
import java.util.ArrayDeque
import java.util.Locale
import java.util.TimeZone
import java.lang.ref.WeakReference

object BlincNativeBridge {

    // Handler type: (args: JSONArray) -> Any?
    private val handlers = mutableMapOf<String, MutableMap<String, (JSONArray) -> Any?>>()

    // Application context for system services
    private var appContext: Context? = null
    private var foregroundActivityRef: WeakReference<Activity>? = null
    private val sensorCollector = AndroidSensorCollector()

    /**
     * Initialize with application context
     */
    fun init(context: Context) {
        appContext = context.applicationContext
        sensorCollector.attach(context.applicationContext)
        sensorCollector.setForegroundActivity(foregroundActivityRef?.get())
    }

    /**
     * Set the currently visible Activity for runtime permission requests.
     */
    fun setForegroundActivity(activity: Activity?) {
        foregroundActivityRef = if (activity == null) null else WeakReference(activity)
        sensorCollector.setForegroundActivity(activity)
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
            val style = args.optInt(0, 1)
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
            sensorCollector.hasLocationPermission()
        }
        register("permissions", "has_motion") {
            sensorCollector.hasMotionPermission()
        }
        register("permissions", "request_location_when_in_use") {
            sensorCollector.requestLocationPermissionWhenInUse()
        }
        register("permissions", "request_location_always") {
            sensorCollector.requestLocationPermissionAlways()
        }
        register("permissions", "request_motion") {
            sensorCollector.requestMotionPermission()
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

    fun hasMotionPermission(): Boolean {
        val ctx = context ?: return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return true
        }
        return ctx.checkSelfPermission(Manifest.permission.ACTIVITY_RECOGNITION) ==
            PackageManager.PERMISSION_GRANTED
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
        val activity = foregroundActivityRef?.get() ?: return false
        activity.runOnUiThread {
            activity.requestPermissions(
                arrayOf(Manifest.permission.ACTIVITY_RECOGNITION),
                REQUEST_CODE_MOTION,
            )
        }
        return false
    }

    companion object {
        private const val TAG = "BlincSensor"
        private const val STATS_LOG_INTERVAL_MS = 1_000L
        private const val REQUEST_CODE_LOCATION_WHEN_IN_USE = 3101
        private const val REQUEST_CODE_LOCATION_ALWAYS = 3102
        private const val REQUEST_CODE_MOTION = 3103
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
        )
    }

    private fun appendFrame(kind: String, monotonicNs: Long, accuracy: String, values: FloatArray) {
        var statsLog: String? = null
        synchronized(lock) {
            if (!running) {
                return
            }
            val frame = JSONObject()
            frame.put("seq", ++seq)
            frame.put("sensor", kind)
            frame.put("time_monotonic_ns", monotonicNs)
            frame.put("time_unix_ms", monotonicToUnixMs(monotonicNs))
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
            latestSampleByKind[kind] = formatValues(values)

            val nowMs = SystemClock.elapsedRealtime()
            if (nowMs - lastStatsLogMs >= STATS_LOG_INTERVAL_MS) {
                lastStatsLogMs = nowMs
                val counts = frameCountsByKind
                    .entries
                    .sortedBy { it.key }
                    .joinToString(", ") { "${it.key}=${it.value}" }
                val latest = listOf("gps", "accelerometer", "gyroscope", "magnetometer", "barometer")
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

private data class Quadruple<A, B, C, D>(
    val first: A,
    val second: B,
    val third: C,
    val fourth: D,
)
