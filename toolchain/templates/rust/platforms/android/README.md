# Android Platform Setup

## Prerequisites

- Android Studio (latest)
- Android NDK 26.1+ (install via SDK Manager)
- Rust with Android targets:
  ```bash
  rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
  ```
- cargo-ndk: `cargo install cargo-ndk`

## Environment Setup

Set the NDK path:
```bash
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.1.10909125
```

## Building

### Debug Build

```bash
# From project root
cargo ndk -t arm64-v8a build --lib

# Then build APK
cd platforms/android
./gradlew assembleDebug
```

### Release Build

```bash
cargo ndk -t arm64-v8a build --lib --release
cd platforms/android
./gradlew assembleRelease
```

## Running

```bash
# Install and run on connected device
cd platforms/android
./gradlew installDebug
adb shell am start -n com.blinc.{{project_name_snake}}/.MainActivity
```

## Project Structure

```
platforms/android/
├── app/
│   ├── src/main/
│   │   ├── kotlin/com/blinc/
│   │   │   ├── MainActivity.kt      # Android entry point
│   │   │   └── BlincNativeBridge.kt # Rust-to-Kotlin bridge
│   │   ├── jniLibs/                  # Rust .so files (auto-copied)
│   │   └── AndroidManifest.xml
│   └── build.gradle.kts
├── build.gradle.kts
└── settings.gradle.kts
```

## Native Bridge

The `BlincNativeBridge` allows Rust to call Kotlin functions:

```rust
// In Rust
let battery: String = native_call("device", "get_battery_level", ()).unwrap();
```

```kotlin
// In Kotlin (already registered by default)
BlincNativeBridge.registerString("device", "get_battery_level") {
    // Return battery percentage as string
}
```
