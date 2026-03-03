# example

A Blinc UI application with cross-platform support for desktop, Android, and iOS.

## Quick Start

### Desktop

```bash
cargo run --features desktop
```

### Android

```bash
# Build + install on connected device/emulator
./build-android.sh
```

The script requests runtime permissions needed for sensors and platform services:
`LOCATION`, `ACTIVITY_RECOGNITION`, `MICROPHONE`, and `BLUETOOTH` (Android 12+).

### iOS

```bash
# Build Rust static libraries for device/simulator
./build-ios.sh

# Open Xcode project and run
open platforms/ios/BlincApp.xcodeproj
```

### Verify Mobile Builds

```bash
# Ensure native bridge templates and example files are synced
../../scripts/check-mobile-native-bridge-sync.sh

# Android + iOS end-to-end build verification
./test-mobile-builds.sh
```

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
