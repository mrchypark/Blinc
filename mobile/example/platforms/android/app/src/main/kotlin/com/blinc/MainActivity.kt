package com.blinc.example

import android.app.NativeActivity
import android.os.Bundle
import com.blinc.BlincNativeBridge

class MainActivity : NativeActivity() {
    companion object {
        init {
            System.loadLibrary("example")
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
