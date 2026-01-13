#!/bin/bash
# Build script for Android mobile example

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Set Java 17 for Android Gradle Plugin compatibility
export JAVA_HOME=/opt/homebrew/opt/openjdk@17

# Set Android SDK paths
export ANDROID_HOME=~/Library/Android/sdk
ADB="$ANDROID_HOME/platform-tools/adb"

# Step 1: Build Rust library for Android
echo "Building Rust library for arm64-v8a..."
cargo ndk -t arm64-v8a -o platforms/android/app/src/main/jniLibs build --release

# Step 2: Build APK
echo "Building APK..."
cd platforms/android
./gradlew assembleDebug

# Step 3: Install APK (if device connected)
if $ADB devices | grep -q "device$"; then
    echo "Installing APK..."
    $ADB install -r app/build/outputs/apk/debug/app-debug.apk

    # Step 4: Start the app
    echo "Starting app..."
    $ADB shell am start -n com.blinc.example/.MainActivity

    # Step 5: Show logs
    echo "Showing logs (Ctrl+C to exit)..."
    $ADB logcat -c  # Clear old logs
    $ADB logcat -s Blinc:D RustStdoutStderr:D
else
    echo "No device connected. APK is at:"
    echo "  platforms/android/app/build/outputs/apk/debug/app-debug.apk"
fi
