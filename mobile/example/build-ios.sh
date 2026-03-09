#!/bin/bash
# Build script for iOS mobile example

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Project configuration
PROJECT_NAME="BlincApp"
BUNDLE_ID="com.blinc.example"
LIB_NAME="libexample.a"
PROJECT_PATH="$SCRIPT_DIR/platforms/ios/${PROJECT_NAME}.xcodeproj"
SCHEME_NAME="${BLINC_IOS_SCHEME:-$PROJECT_NAME}"
ARTIFACT_ROOT="$SCRIPT_DIR/artifacts/ios"
ARCHIVE_PATH_DEFAULT="$ARTIFACT_ROOT/archive/${PROJECT_NAME}.xcarchive"
EXPORT_PATH_DEFAULT="$ARTIFACT_ROOT/export"

# iOS targets
TARGET_ARM64="aarch64-apple-ios"
TARGET_SIM_ARM64="aarch64-apple-ios-sim"
TARGET_SIM_X86="x86_64-apple-ios"

usage() {
    cat <<'EOF'
Usage: ./build-ios.sh [debug-libs|release-libs|archive|export-archive]

Modes:
  debug-libs      Build Rust iOS static libraries for local debug runs.
  release-libs    Build Rust iOS static libraries for release packaging.
  archive         Build release Rust libs, then run xcodebuild archive.
  export-archive  Export an existing archive to an .ipa or xcarchive export directory.

Environment:
  BLINC_IOS_SCHEME                  Xcode scheme (default: BlincApp)
  BLINC_IOS_CONFIGURATION           Xcode configuration (default: Release)
  BLINC_IOS_DESTINATION             Xcode destination (default: generic/platform=iOS)
  BLINC_IOS_ARCHIVE_PATH            Archive output path (default: artifacts/ios/archive/BlincApp.xcarchive)
  BLINC_IOS_EXPORT_PATH             Export output path (default: artifacts/ios/export)
  BLINC_IOS_EXPORT_OPTIONS_PLIST    Required for export-archive
  BLINC_IOS_TEAM_ID                 Optional development team
  BLINC_IOS_CODE_SIGNING_ALLOWED    YES/NO; default NO for archive automation
EOF
}

MODE="${1:-debug-libs}"
case "$MODE" in
    debug-libs)
        CARGO_FLAGS=""
        TARGET_DIR="debug"
        LIBS_DIR="$ARTIFACT_ROOT/libs/debug"
        ;;
    release-libs|archive|export-archive)
        CARGO_FLAGS="--release"
        TARGET_DIR="release"
        LIBS_DIR="$ARTIFACT_ROOT/libs/release"
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

# Ensure iOS targets are installed
echo "Checking Rust iOS targets..."
if ! rustup target list --installed | grep -q "$TARGET_ARM64"; then
    echo "Installing $TARGET_ARM64..."
    rustup target add "$TARGET_ARM64"
fi

if ! rustup target list --installed | grep -q "$TARGET_SIM_ARM64"; then
    echo "Installing $TARGET_SIM_ARM64..."
    rustup target add "$TARGET_SIM_ARM64"
fi
if ! rustup target list --installed | grep -q "$TARGET_SIM_X86"; then
    echo "Installing $TARGET_SIM_X86..."
    rustup target add "$TARGET_SIM_X86"
fi

build_rust_libraries() {
    echo ""
    echo "=== Building Rust static library (${MODE}) ==="
    cd "$SCRIPT_DIR"

    echo "Building for device ($TARGET_ARM64)..."
    cargo build --lib --features ios $CARGO_FLAGS --target "$TARGET_ARM64"

    echo "Building for simulator ($TARGET_SIM_ARM64)..."
    cargo build --lib --features ios $CARGO_FLAGS --target "$TARGET_SIM_ARM64"

    echo "Building for simulator ($TARGET_SIM_X86)..."
    cargo build --lib --features ios $CARGO_FLAGS --target "$TARGET_SIM_X86"

    mkdir -p "$LIBS_DIR/device"
    mkdir -p "$LIBS_DIR/simulator"

    echo "Copying libraries..."
    cp "target/$TARGET_ARM64/$TARGET_DIR/$LIB_NAME" "$LIBS_DIR/device/"
    cp "target/$TARGET_SIM_ARM64/$TARGET_DIR/$LIB_NAME" "$LIBS_DIR/simulator/$LIB_NAME.arm64"
    cp "target/$TARGET_SIM_X86/$TARGET_DIR/$LIB_NAME" "$LIBS_DIR/simulator/$LIB_NAME.x86_64"

    echo "Creating universal simulator library..."
    lipo -create \
        "$LIBS_DIR/simulator/$LIB_NAME.arm64" \
        "$LIBS_DIR/simulator/$LIB_NAME.x86_64" \
        -output "$LIBS_DIR/simulator/$LIB_NAME"
    rm -f "$LIBS_DIR/simulator/$LIB_NAME.arm64" "$LIBS_DIR/simulator/$LIB_NAME.x86_64"

    echo ""
    echo "Libraries are at:"
    echo "  Device:    $LIBS_DIR/device/$LIB_NAME"
    echo "  Simulator: $LIBS_DIR/simulator/$LIB_NAME"
}

archive_app() {
    local configuration="${BLINC_IOS_CONFIGURATION:-Release}"
    local destination="${BLINC_IOS_DESTINATION:-generic/platform=iOS}"
    local archive_path="${BLINC_IOS_ARCHIVE_PATH:-$ARCHIVE_PATH_DEFAULT}"
    local code_signing_allowed="${BLINC_IOS_CODE_SIGNING_ALLOWED:-NO}"

    if [ ! -d "$PROJECT_PATH" ]; then
        echo "Missing iOS Xcode project: $PROJECT_PATH" >&2
        exit 1
    fi
    if ! command -v xcodebuild >/dev/null 2>&1; then
        echo "xcodebuild is required for archive mode." >&2
        exit 1
    fi

    mkdir -p "$(dirname "$archive_path")"
    build_rust_libraries

    echo ""
    echo "=== Archiving iOS app ==="
    XCODE_ARGS=(
        -project "$PROJECT_PATH"
        -scheme "$SCHEME_NAME"
        -configuration "$configuration"
        -destination "$destination"
        -archivePath "$archive_path"
        archive
        CODE_SIGNING_ALLOWED="$code_signing_allowed"
    )
    if [ -n "${BLINC_IOS_TEAM_ID:-}" ]; then
        XCODE_ARGS+=("DEVELOPMENT_TEAM=${BLINC_IOS_TEAM_ID}")
    fi
    xcodebuild "${XCODE_ARGS[@]}"

    echo "Archive exported to:"
    echo "  $archive_path"
}

export_archive() {
    local archive_path="${BLINC_IOS_ARCHIVE_PATH:-$ARCHIVE_PATH_DEFAULT}"
    local export_path="${BLINC_IOS_EXPORT_PATH:-$EXPORT_PATH_DEFAULT}"
    local export_options_plist="${BLINC_IOS_EXPORT_OPTIONS_PLIST:-}"

    if [ -z "$export_options_plist" ]; then
        echo "BLINC_IOS_EXPORT_OPTIONS_PLIST is required for export-archive." >&2
        exit 1
    fi
    if [ ! -d "$archive_path" ]; then
        echo "Archive does not exist: $archive_path" >&2
        exit 1
    fi
    if ! command -v xcodebuild >/dev/null 2>&1; then
        echo "xcodebuild is required for export-archive mode." >&2
        exit 1
    fi

    mkdir -p "$export_path"
    echo ""
    echo "=== Exporting iOS archive ==="
    xcodebuild -exportArchive \
        -archivePath "$archive_path" \
        -exportPath "$export_path" \
        -exportOptionsPlist "$export_options_plist"

    echo "Exported iOS artifacts to:"
    echo "  $export_path"
}

case "$MODE" in
    debug-libs|release-libs)
        build_rust_libraries
        echo ""
        echo "Next steps:"
        echo "  1. Open platforms/ios/$PROJECT_NAME.xcodeproj in Xcode"
        echo "  2. Select your target device/simulator"
        echo "  3. Build and run (Cmd+R)"
        ;;
    archive)
        archive_app
        ;;
    export-archive)
        export_archive
        ;;
esac
