#!/bin/bash
# Build script for Android mobile example

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

if [ ! -f "$OUTPUT_PATH" ] && [ -n "${FALLBACK_OUTPUT_PATH:-}" ] && [ -f "$FALLBACK_OUTPUT_PATH" ]; then
    OUTPUT_PATH="$FALLBACK_OUTPUT_PATH"
fi

if [ ! -f "$OUTPUT_PATH" ]; then
    echo "Expected Android artifact was not produced: $OUTPUT_PATH" >&2
    exit 1
fi

cp "$OUTPUT_PATH" "$ARTIFACT_DIR/$ARTIFACT_NAME"
echo "Exported Android artifact:"
echo "  $ARTIFACT_DIR/$ARTIFACT_NAME"

# Step 3: Install APK (debug only)
if [ "$INSTALL_ON_DEVICE" -eq 1 ]; then
    DEVICE_SERIAL="$($ADB devices | awk 'NR>1 && $2=="device" {print $1; exit}')"
    if [ -n "$DEVICE_SERIAL" ]; then
        echo "Installing APK..."
        "$ADB" -s "$DEVICE_SERIAL" install -r "$OUTPUT_PATH"
        grant_runtime_permissions "$DEVICE_SERIAL"

        echo "Starting app..."
        "$ADB" -s "$DEVICE_SERIAL" shell am start -n "${PACKAGE_NAME}/${ACTIVITY_NAME}"

        echo "Showing logs (Ctrl+C to exit)..."
        "$ADB" -s "$DEVICE_SERIAL" logcat -c
        "$ADB" -s "$DEVICE_SERIAL" logcat | grep --line-buffered -E "Sensor permissions requested|Supported mobile sensors|Sensor batch #|Mobile sensors started|Mobile sensors stopped|Blinc|RustStdoutStderr"
    else
        echo "No device connected. Debug APK is at:"
        echo "  $ARTIFACT_DIR/$(basename "$OUTPUT_PATH")"
        echo "Verify with: $ADB devices -l"
    fi
elif [ "$MODE" = "release" ] || [ "$MODE" = "bundle-release" ]; then
    echo "Release packaging finished."
    echo "If signing is required, provide BLINC_ANDROID_KEYSTORE_* inputs before distribution."
fi
