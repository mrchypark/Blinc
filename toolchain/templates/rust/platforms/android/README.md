# Android Platform Setup

For the repo-wide support contract, see
[`docs/native-readiness.md`](../../../../../docs/native-readiness.md).

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

This template currently guarantees scaffold-level Android integration:

- Tier 1: Rust library build + Gradle project generation
- Tier 2: app launch and native bridge integration after project-specific wiring
- Deferred: production signing, store publishing, full mobile accessibility parity

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
│   │   ├── kotlin/com/blinc/        # App-specific sources added after scaffold generation
│   │   ├── jniLibs/                  # Rust .so files (auto-copied)
│   │   └── AndroidManifest.xml
│   └── build.gradle.kts
├── build.gradle.kts
└── settings.gradle.kts
```

## Native Bridge

The generated template includes `BlincNativeBridge.kt` and wires it from
`MainActivity` so common platform helpers are available immediately. Rust can
call Kotlin handlers through that bridge:

```rust
// In Rust
let battery: String = native_call("device", "get_battery_level", ()).unwrap();
```

```kotlin
// In Kotlin (already registered by MainActivity)
BlincNativeBridge.registerString("device", "get_battery_level") {
    // Return battery percentage as string
}
```

`BlincNativeBridge.kt` is maintained alongside the native platform extensions and
may need to be copied into generated projects as the scaffold evolves. If your
generated project does not yet include it, treat the template output as
incomplete scaffold wiring rather than production-ready mobile support.
