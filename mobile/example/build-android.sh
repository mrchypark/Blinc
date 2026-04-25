#!/bin/bash
# Build script for Android mobile example
#
# Usage:
#   bash build-android.sh                  # auto-pick first authorized device
#   ANDROID_SERIAL=emulator-5554 bash …    # target a specific device
#   bash build-android.sh -s <serial>      # ditto, via positional arg

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Prefer caller-provided Java/Android toolchains; fall back to common local defaults.
if [ -z "${JAVA_HOME:-}" ] && [ -d /opt/homebrew/opt/openjdk@17 ]; then
    export JAVA_HOME=/opt/homebrew/opt/openjdk@17
fi

export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
ADB="$ANDROID_HOME/platform-tools/adb"
PACKAGE_NAME="com.blinc.example"
ACTIVITY_NAME=".MainActivity"
ARTIFACT_ROOT="$SCRIPT_DIR/artifacts/android"
DEFAULT_ABIS="${BLINC_ANDROID_ABIS:-arm64-v8a}"

usage() {
    cat <<'EOF'
Usage: ./build-android.sh [debug|release|bundle-release]

Modes:
  debug           Build, install, and launch a debug APK on a connected device.
  release         Build a release APK and export it under artifacts/android/release-apk/.
  bundle-release  Build an Android App Bundle (.aab) and export it under artifacts/android/release-bundle/.

Environment:
  BLINC_ANDROID_ABIS                  Space-separated ABI list for cargo-ndk (default: "arm64-v8a")
  BLINC_ANDROID_SKIP_INSTALL=1        Skip device install/launch even for debug builds
  BLINC_ANDROID_KEYSTORE_PATH         Optional keystore for signed release builds
  BLINC_ANDROID_KEYSTORE_PASSWORD     Optional keystore password
  BLINC_ANDROID_KEY_ALIAS             Optional key alias
  BLINC_ANDROID_KEY_PASSWORD          Optional key password
EOF
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Missing required command: $1" >&2
        exit 1
    fi
}

grant_runtime_permissions() {
    local serial="$1"
    echo "Granting runtime permissions on ${serial}..."
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.ACCESS_FINE_LOCATION || true
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.ACCESS_COARSE_LOCATION || true
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.ACTIVITY_RECOGNITION || true
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.RECORD_AUDIO || true
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.BLUETOOTH_SCAN || true
    "$ADB" -s "$serial" shell pm grant "$PACKAGE_NAME" android.permission.BLUETOOTH_CONNECT || true
}

MODE="${1:-debug}"
case "$MODE" in
    debug)
        GRADLE_TASK="assembleDebug"
        RUST_FLAGS=""
        OUTPUT_PATH="platforms/android/app/build/outputs/apk/debug/app-debug.apk"
        ARTIFACT_DIR="$ARTIFACT_ROOT/debug-apk"
        ARTIFACT_NAME="app-debug.apk"
        INSTALL_ON_DEVICE=1
        ;;
    release)
        GRADLE_TASK="assembleRelease"
        RUST_FLAGS="--release"
        OUTPUT_PATH="platforms/android/app/build/outputs/apk/release/app-release.apk"
        FALLBACK_OUTPUT_PATH="platforms/android/app/build/outputs/apk/release/app-release-unsigned.apk"
        ARTIFACT_DIR="$ARTIFACT_ROOT/release-apk"
        ARTIFACT_NAME="app-release.apk"
        INSTALL_ON_DEVICE=0
        ;;
    bundle-release)
        GRADLE_TASK="bundleRelease"
        RUST_FLAGS="--release"
        OUTPUT_PATH="platforms/android/app/build/outputs/bundle/release/app-release.aab"
        ARTIFACT_DIR="$ARTIFACT_ROOT/release-bundle"
        ARTIFACT_NAME="app-release.aab"
        INSTALL_ON_DEVICE=0
        ;;
    -h|--help|help)
        usage
        exit 0
        ;;
    *)
        echo "Unknown mode: $MODE" >&2
        usage
        exit 1
        ;;
esac

if [ "${BLINC_ANDROID_SKIP_INSTALL:-0}" = "1" ]; then
    INSTALL_ON_DEVICE=0
fi

require_command cargo
require_command cargo-ndk
require_command ./platforms/android/gradlew

mkdir -p "$ARTIFACT_DIR"

if [ -n "${BLINC_ANDROID_KEYSTORE_PATH:-}" ]; then
    export BLINC_ANDROID_KEYSTORE_PATH
    export BLINC_ANDROID_KEYSTORE_PASSWORD="${BLINC_ANDROID_KEYSTORE_PASSWORD:-}"
    export BLINC_ANDROID_KEY_ALIAS="${BLINC_ANDROID_KEY_ALIAS:-}"
    export BLINC_ANDROID_KEY_PASSWORD="${BLINC_ANDROID_KEY_PASSWORD:-}"
    echo "Configured Android signing inputs from BLINC_ANDROID_* environment variables."
fi

# Pin NDK to r29: NDK r28+ links arm64-v8a .so files with 16 KB ELF
# segment alignment by default, which is required for installs on
# Android 16 / Pixel 10 Pro. r27 still uses 4 KB segments and produces
# APKs that fail with "Uncompressed library not aligned" on those
# devices. cargo-ndk auto-discovers the *highest* NDK under
# `$ANDROID_HOME/ndk/`, but pinning explicitly avoids accidents when
# new NDKs land.
if [ -z "$ANDROID_NDK_HOME" ]; then
    if [ -d "$ANDROID_HOME/ndk/29.0.14206865" ]; then
        export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
    fi
fi
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export NDK_HOME="$ANDROID_NDK_HOME"

# Allow `-s <serial>` override on the command line
if [ "$1" = "-s" ] && [ -n "$2" ]; then
    export ANDROID_SERIAL="$2"
fi

# Resolve target device:
#   1. ANDROID_SERIAL env var if set (and authorized)
#   2. Otherwise the first `device` (authorized) entry from `adb devices`
#      — unauthorized / offline / no_permissions entries are skipped.
if [ -n "$ANDROID_SERIAL" ]; then
    DEVICE_SERIAL="$ANDROID_SERIAL"
else
    DEVICE_SERIAL=$($ADB devices | awk '$2 == "device" { print $1; exit }')
fi

if [ -z "$DEVICE_SERIAL" ]; then
    echo "No authorized device connected. Plug one in and accept the USB debugging prompt."
    echo "(APK will still be built and left at platforms/android/app/build/outputs/apk/debug/app-debug.apk)"
fi

# Step 1: Build Rust library for Android
echo "Building Rust library for Android ABIs: ${DEFAULT_ABIS} (${MODE})..."
ABI_ARGS=()
for abi in $DEFAULT_ABIS; do
    ABI_ARGS+=("-t" "$abi")
done
cargo ndk "${ABI_ARGS[@]}" -o platforms/android/app/src/main/jniLibs build $RUST_FLAGS

# Step 2: Build Gradle artifact
echo "Building APK via ${GRADLE_TASK}..."
cd platforms/android
./gradlew "${GRADLE_TASK}"
cd "$SCRIPT_DIR"

if [ -n "$DEVICE_SERIAL" ]; then
    echo "Installing APK to $DEVICE_SERIAL..."
    $ADB -s "$DEVICE_SERIAL" install -r app/build/outputs/apk/debug/app-debug.apk

    echo "Starting app on $DEVICE_SERIAL..."
    $ADB -s "$DEVICE_SERIAL" shell am start -n com.blinc.example/.MainActivity

    echo "Showing logs (Ctrl+C to exit)..."
    $ADB -s "$DEVICE_SERIAL" logcat -c  # Clear old logs
    $ADB -s "$DEVICE_SERIAL" logcat -s Blinc:D RustStdoutStderr:D AndroidRuntime:E DEBUG:F BlincNativeBridge:D
else
    echo "APK is at:"
    echo "  platforms/android/app/build/outputs/apk/debug/app-debug.apk"
fi
