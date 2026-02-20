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
PACKAGE_NAME="com.blinc.example"
ACTIVITY_NAME=".MainActivity"

grant_runtime_permissions() {
    local serial="$1"
    echo "Granting runtime permissions on ${serial}..."
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.ACCESS_FINE_LOCATION || true
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.ACCESS_COARSE_LOCATION || true
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.ACTIVITY_RECOGNITION || true
}

# Step 1: Build Rust library for Android
echo "Building Rust library for arm64-v8a..."
cargo ndk -t arm64-v8a -o platforms/android/app/src/main/jniLibs build --release

# Step 2: Build APK
echo "Building APK..."
cd platforms/android
./gradlew assembleDebug

# Step 3: Install APK (if device connected)
DEVICE_SERIAL="$($ADB devices | awk 'NR>1 && $2=="device" {print $1; exit}')"
if [ -n "$DEVICE_SERIAL" ]; then
    echo "Installing APK..."
    $ADB -s "$DEVICE_SERIAL" install -r app/build/outputs/apk/debug/app-debug.apk
    grant_runtime_permissions "$DEVICE_SERIAL"

    # Step 4: Start the app
    echo "Starting app..."
    $ADB -s "$DEVICE_SERIAL" shell am start -n "${PACKAGE_NAME}/${ACTIVITY_NAME}"

    # Step 5: Show logs
    echo "Showing logs (Ctrl+C to exit)..."
    $ADB -s "$DEVICE_SERIAL" logcat -c  # Clear old logs
    $ADB -s "$DEVICE_SERIAL" logcat | grep --line-buffered -E "Sensor permissions requested|Supported mobile sensors|Sensor batch #|Mobile sensors started|Mobile sensors stopped|Blinc|RustStdoutStderr"
else
    echo "No device connected. APK is at:"
    echo "  platforms/android/app/build/outputs/apk/debug/app-debug.apk"
    echo ""
    echo "Tips:"
    echo "  1. Enable USB debugging on the Android phone."
    echo "  2. Accept the host RSA prompt on the device."
    echo "  3. Verify with: $ADB devices -l"
fi
