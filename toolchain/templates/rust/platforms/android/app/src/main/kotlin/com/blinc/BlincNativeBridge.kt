/**
 * Blinc Native Bridge for Android
 *
 * Kotlin implementation for handling native calls from Rust.
 * Register handlers for each namespace/function, then Rust can call
 * them via native_call("namespace", "function", args).
 */

package com.blinc.{{project_name_snake}}

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.BatteryManager
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import androidx.core.content.getSystemService
import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale
import java.util.TimeZone

object BlincNativeBridge {

    private val handlers = mutableMapOf<String, MutableMap<String, (JSONArray) -> Any?>>()
    private var appContext: Context? = null

    fun init(context: Context) {
        appContext = context.applicationContext
    }

    fun register(namespace: String, name: String, handler: (JSONArray) -> Any?) {
        handlers.getOrPut(namespace) { mutableMapOf() }[name] = handler
    }

    fun registerString(namespace: String, name: String, handler: () -> String) {
        register(namespace, name) { handler() }
    }

    fun registerVoid(namespace: String, name: String, handler: () -> Unit) {
        register(namespace, name) { handler(); null }
    }

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

    fun registerDefaults(context: Context) {
        init(context)
        val ctx = context.applicationContext

        // Device namespace
        registerString("device", "get_battery_level") {
            val bm = ctx.getSystemService<BatteryManager>()
            bm?.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)?.toString() ?: "0"
        }

        registerString("device", "get_model") { Build.MODEL }
        registerString("device", "get_os_version") { Build.VERSION.RELEASE }

        register("device", "is_low_power_mode") {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                val pm = ctx.getSystemService(Context.POWER_SERVICE) as? android.os.PowerManager
                pm?.isPowerSaveMode ?: false
            } else false
        }

        registerString("device", "get_locale") { Locale.getDefault().toString() }
        registerString("device", "get_timezone") { TimeZone.getDefault().id }

        // Haptics namespace
        register("haptics", "vibrate") { args ->
            val durationMs = args.optLong(0, 100)
            vibrate(ctx, durationMs)
            null
        }

        register("haptics", "impact") { args ->
            val style = args.optInt(0, 1)
            val amplitude = when (style) { 0 -> 50; 2 -> 255; else -> 128 }
            vibrateWithAmplitude(ctx, 10, amplitude)
            null
        }

        registerVoid("haptics", "selection") { vibrateWithAmplitude(ctx, 5, 50) }
        registerVoid("haptics", "success") { vibrateWithAmplitude(ctx, 30, 200) }
        registerVoid("haptics", "warning") { vibrateWithAmplitude(ctx, 50, 150) }
        registerVoid("haptics", "error") { vibrateWithAmplitude(ctx, 100, 255) }

        // Clipboard namespace
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

        // App namespace
        registerString("app", "get_version") {
            try {
                ctx.packageManager.getPackageInfo(ctx.packageName, 0).versionName ?: "1.0"
            } catch (e: Exception) { "1.0" }
        }

        registerString("app", "get_bundle_id") { ctx.packageName }

        register("app", "open_url") { args ->
            val url = args.optString(0, "")
            try {
                val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
                intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                ctx.startActivity(intent)
                true
            } catch (e: Exception) { false }
        }
    }

    private fun successJson(value: Any?): String {
        val obj = JSONObject()
        obj.put("success", true)
        when (value) {
            null -> obj.put("value", JSONObject.NULL)
            is String -> obj.put("value", value)
            is Boolean -> obj.put("value", value)
            is Number -> obj.put("value", value)
            else -> obj.put("value", value.toString())
        }
        return obj.toString()
    }

    private fun errorJson(type: String, message: String): String {
        return JSONObject().apply {
            put("success", false)
            put("errorType", type)
            put("errorMessage", message)
        }.toString()
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
