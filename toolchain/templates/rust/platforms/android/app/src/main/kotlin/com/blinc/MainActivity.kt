package com.blinc.{{project_name_snake}}

import android.app.NativeActivity
import android.os.Bundle

/**
 * Main Activity for {{project_name}}
 *
 * This activity loads the Rust library and delegates to the native code.
 * The actual UI is rendered by Blinc via the native library.
 */
class MainActivity : NativeActivity() {
    companion object {
        init {
            // Load the Rust library
            System.loadLibrary("{{project_name_snake}}")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        BlincNativeBridge.setForegroundActivity(this)
        BlincNativeBridge.registerDefaults(applicationContext)
        super.onCreate(savedInstanceState)
    }

    override fun onResume() {
        super.onResume()
        BlincNativeBridge.setForegroundActivity(this)
    }

    override fun onPause() {
        BlincNativeBridge.setForegroundActivity(null)
        super.onPause()
    }

    override fun onDestroy() {
        BlincNativeBridge.setForegroundActivity(null)
        super.onDestroy()
    }
}
