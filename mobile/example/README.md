# example

A Blinc UI application with cross-platform support for desktop, Android, and iOS.

## Quick Start

### Desktop

```bash
cargo run --features desktop
```

### Android

```bash
# Build Rust library
cargo ndk -t arm64-v8a build --lib

# Build and install APK
cd platforms/android
./gradlew installDebug
```

### iOS

```bash
# Build Rust library
cargo lipo --release

# Open Xcode project and run
```

## Project Structure

```
example/
├── Cargo.toml           # Rust project configuration
├── blinc.toml           # Blinc toolchain configuration
├── src/
│   └── main.rs          # Application code
└── platforms/
    ├── android/         # Android Gradle project
    └── ios/             # iOS Swift files
```
