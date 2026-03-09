# blinc_platform_android

> **Part of the [Blinc UI Framework](https://project-blinc.github.io/Blinc)**
>
> This crate is a component of Blinc, a GPU-accelerated UI framework for Rust.
> For full documentation and guides, visit the [Blinc documentation](https://project-blinc.github.io/Blinc).

Android platform scaffolding for Blinc UI.

## Overview

`blinc_platform_android` provides the current Android runtime surface for Blinc:
NativeActivity entry points, JNI/native bridge helpers, touch forwarding, asset
loading, and the renderer integration points used by generated templates.

## Supported Platforms

- Android 7.0+ (API level 24+)
- ARM64 and x86_64 architectures

## Features

- **Native Activity**: Android entry-point helpers
- **JNI Bridge**: Rust-to-Kotlin interoperability
- **Touch Input**: Touch forwarding into Blinc events
- **Asset Loading**: Load from APK resources
- **Template Support**: Kotlin bridge wiring for generated projects

## Quick Start

```rust
use blinc_platform_android::android_main;

#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    saved_state: *mut c_void,
    saved_state_size: usize,
) {
    android_main(activity, |ctx| {
        // Build your UI
        div()
            .w_full()
            .h_full()
            .child(text("Hello Android!"))
    });
}
```

## Status

The Android runtime is still under active development. Use it as template-backed
scaffolding for shared UI experiments and app integration, and verify renderer
and lifecycle behavior in your target app before treating it as production
ready.

## Project Setup

### Cargo.toml

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
blinc_platform_android = "0.1"
```

### build.gradle

```gradle
android {
    defaultConfig {
        ndk {
            abiFilters 'arm64-v8a', 'x86_64'
        }
    }

    externalNativeBuild {
        ndkBuild {
            path 'src/main/jni/Android.mk'
        }
    }
}
```

### AndroidManifest.xml

```xml
<application
    android:hasCode="false"
    android:allowBackup="true">
    <activity
        android:name="android.app.NativeActivity"
        android:exported="true"
        android:configChanges="orientation|screenSize|keyboardHidden">
        <meta-data
            android:name="android.app.lib_name"
            android:value="myapp" />
        <intent-filter>
            <action android:name="android.intent.action.MAIN" />
            <category android:name="android.intent.category.LAUNCHER" />
        </intent-filter>
    </activity>
</application>
```

## Touch Handling

```rust
fn handle_touch(event: TouchEvent) {
    for pointer in event.pointers() {
        match pointer.action {
            PointerAction::Down => {
                // Touch started
            }
            PointerAction::Move => {
                // Touch moved
            }
            PointerAction::Up => {
                // Touch ended
            }
        }
    }
}
```

## Asset Loading

```rust
use blinc_platform_android::AndroidAssetLoader;

let loader = AndroidAssetLoader::new(activity);

// Load from assets/ directory in APK
let data = loader.load("images/logo.png")?;

// Check if asset exists
if loader.exists("config.json") {
    let config = loader.load("config.json")?;
}
```

## Building

```bash
# Build for Android
cargo ndk -t arm64-v8a -t x86_64 -o ./app/src/main/jniLibs build --release

# Or using cargo-apk
cargo apk build --release
```

## Requirements

- Android SDK (API level 24+)
- Android NDK r21+
- Rust with Android targets:
  ```bash
  rustup target add aarch64-linux-android
  rustup target add x86_64-linux-android
  ```
- cargo-ndk:
  ```bash
  cargo install cargo-ndk
  ```

## License

MIT OR Apache-2.0
