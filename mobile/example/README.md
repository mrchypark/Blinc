# example

A Blinc UI application with cross-platform support for desktop, Android, and iOS.
This is the canonical native reference app for current mobile runtime and bridge work.

See [`docs/native-readiness.md`](../../docs/native-readiness.md)
for the repo-wide support contract and tier definitions.
See [`docs/mobile-release.md`](../../docs/mobile-release.md)
for the debug-vs-release packaging contract and artifact paths.

## Quick Start

### Desktop

```bash
cargo run --features desktop
```

### Android

```bash
# Debug smoke: build + install on connected device/emulator
./build-android.sh debug

# Release packaging: APK
./build-android.sh release

# Release packaging: Android App Bundle
./build-android.sh bundle-release
```

The debug script grants runtime permissions needed for sensors and platform services:
`LOCATION`, `ACTIVITY_RECOGNITION`, `MICROPHONE`, and `BLUETOOTH` (Android 12+).
Release artifacts are exported under `artifacts/android/`.

### iOS

```bash
# Debug smoke: build Rust static libraries for device/simulator
./build-ios.sh debug-libs

# Release packaging: Rust static libraries
./build-ios.sh release-libs

# Release packaging: Xcode archive
./build-ios.sh archive

# Export a previously-created archive (.ipa export requires export options)
BLINC_IOS_EXPORT_OPTIONS_PLIST=/abs/path/ExportOptions.plist ./build-ios.sh export-archive

# Open Xcode project and run
open platforms/ios/BlincApp.xcodeproj
```

Release artifacts are exported under `artifacts/ios/`.

### Verify Mobile Builds

```bash
# Ensure native bridge templates and example files are synced
../../scripts/check-mobile-native-bridge-sync.sh

# Android + iOS end-to-end build verification
./test-mobile-builds.sh
```

Current support tiers for this example:

- Tier 1: cross-target build, native bridge sync, simulator/device debug runs
- Tier 2: runtime feature validation for sensors, permissions, clipboard/share/haptics bridge paths
- Packaging progress: release APK/AAB export, iOS archive/export command contract, artifact path stability
- Deferred: store submission, notarization, and app-store-specific release automation

### Verify Runtime Sensors On Device

```bash
# Android device/emulator log monitoring
./test-sensor-runtime.sh android

# iOS simulator log monitoring
./test-sensor-runtime.sh ios-sim
```

### Drive Android Emulator Sensors (0.5s Loop)

```bash
# Print emulator-supported virtual sensors
./simulate-android-sensors.sh --list-only

# Drive booted emulator location + IMU sensors every 0.5s
./simulate-android-sensors.sh --interval 0.5

# With explicit emulator serial
./simulate-android-sensors.sh --device emulator-5554 --interval 0.5
```

For Android verification logs, watch:

- `adb -s <device> logcat -s BlincSensor:V Blinc:V *:S`

### Drive iOS Simulator Sensors (0.5s Location Loop)

```bash
# Drive booted simulator location every 0.5s (simctl location set loop)
./simulate-ios-sensors.sh --interval 0.5

# With explicit device UDID
./simulate-ios-sensors.sh --device 6D96849D-E6A4-4628-940A-9EFED1BD0829 --interval 0.5
```

This helper grants app permissions, optionally launches the app, and drives
simulated GPS updates continuously. It also prints a sensor support matrix:

- Simulator-drivable: `GPS`, `Heading`
- Requires real device: `Accelerometer`, `Gyroscope`, `Magnetometer`, `Barometer`, `Step/Cadence`, `Activity`

Expected runtime log markers after launching the app and granting permissions:

- `Sensor permissions requested: ...`
- `Supported mobile sensors: [...]`
- `Sensor batch #...: frames=... kinds=[Gps=..., Accelerometer=..., ...]`

For physical iOS devices, run from Xcode and inspect the app debug console.

## Project Structure

```
example/
├── Cargo.toml           # Rust project configuration
├── blinc.toml           # Blinc toolchain configuration
├── test-mobile-builds.sh # Android+iOS build verification
├── test-sensor-runtime.sh # Runtime sensor verification helper
├── simulate-ios-sensors.sh # iOS simulator sensor driving helper
├── src/
│   └── main.rs          # Application code
└── platforms/
    ├── android/         # Android Gradle project
    └── ios/             # iOS Swift files
```
