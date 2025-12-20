#!/bin/bash
# Android cross-compilation environment setup
# Source this file before building for Android:
#   source scripts/android-env.sh
#
# Supported platforms: macOS, Linux, Windows (via Git Bash/MSYS2/WSL)

# Detect Android SDK location
if [ -z "$ANDROID_SDK_HOME" ]; then
    # macOS locations
    if [ -d "$HOME/Library/Android/sdk" ]; then
        export ANDROID_SDK_HOME="$HOME/Library/Android/sdk"
    # Linux locations
    elif [ -d "$HOME/Android/Sdk" ]; then
        export ANDROID_SDK_HOME="$HOME/Android/Sdk"
    elif [ -d "/opt/android-sdk" ]; then
        export ANDROID_SDK_HOME="/opt/android-sdk"
    elif [ -d "/usr/local/android-sdk" ]; then
        export ANDROID_SDK_HOME="/usr/local/android-sdk"
    # Windows locations (Git Bash / MSYS2)
    elif [ -d "/c/Users/$USER/AppData/Local/Android/Sdk" ]; then
        export ANDROID_SDK_HOME="/c/Users/$USER/AppData/Local/Android/Sdk"
    elif [ -d "$LOCALAPPDATA/Android/Sdk" ]; then
        export ANDROID_SDK_HOME="$LOCALAPPDATA/Android/Sdk"
    # WSL accessing Windows SDK
    elif [ -d "/mnt/c/Users/$USER/AppData/Local/Android/Sdk" ]; then
        export ANDROID_SDK_HOME="/mnt/c/Users/$USER/AppData/Local/Android/Sdk"
    else
        echo "Error: Could not find Android SDK."
        echo "Please set ANDROID_SDK_HOME environment variable manually."
        echo ""
        echo "Common locations:"
        echo "  macOS:   ~/Library/Android/sdk"
        echo "  Linux:   ~/Android/Sdk or /opt/android-sdk"
        echo "  Windows: %LOCALAPPDATA%\\Android\\Sdk"
        return 1
    fi
fi

# Find the latest NDK version
if [ -z "$ANDROID_NDK_HOME" ]; then
    NDK_DIR="$ANDROID_SDK_HOME/ndk"
    if [ -d "$NDK_DIR" ]; then
        # Get the latest NDK version (sort by version number)
        LATEST_NDK=$(ls -1 "$NDK_DIR" 2>/dev/null | sort -V | tail -1)
        if [ -n "$LATEST_NDK" ]; then
            export ANDROID_NDK_HOME="$NDK_DIR/$LATEST_NDK"
        else
            echo "Error: No NDK found in $NDK_DIR"
            echo "Install NDK using: sdkmanager 'ndk;27.0.12077973'"
            return 1
        fi
    else
        echo "Error: NDK directory not found at $NDK_DIR"
        echo "Install NDK using: sdkmanager 'ndk;27.0.12077973'"
        return 1
    fi
fi

# Verify NDK exists
if [ ! -d "$ANDROID_NDK_HOME" ]; then
    echo "Error: NDK not found at $ANDROID_NDK_HOME"
    return 1
fi

# Set up toolchain paths
TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt"

# Detect host platform
case "$(uname -s)" in
    Darwin*)
        if [ "$(uname -m)" = "arm64" ]; then
            HOST_TAG="darwin-arm64"
            # Fallback to x86_64 if arm64 doesn't exist (older NDK)
            [ ! -d "$TOOLCHAIN/$HOST_TAG" ] && HOST_TAG="darwin-x86_64"
        else
            HOST_TAG="darwin-x86_64"
        fi
        ;;
    Linux*)
        HOST_TAG="linux-x86_64"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        HOST_TAG="windows-x86_64"
        ;;
    *)
        echo "Error: Unsupported host platform: $(uname -s)"
        echo "Supported: Darwin (macOS), Linux, Windows (MINGW/MSYS/Cygwin)"
        return 1
        ;;
esac

TOOLCHAIN_BIN="$TOOLCHAIN/$HOST_TAG/bin"

if [ ! -d "$TOOLCHAIN_BIN" ]; then
    echo "Error: Toolchain not found at $TOOLCHAIN_BIN"
    echo "Check that your NDK installation is complete."
    return 1
fi

# Add toolchain to PATH
export PATH="$TOOLCHAIN_BIN:$PATH"

# Minimum Android API level (Android 12 = API 31, but we use 35 for latest features)
# Adjust based on your target requirements
API_LEVEL=35

# Set CC/CXX for build scripts that need them
export CC_aarch64_linux_android="$TOOLCHAIN_BIN/aarch64-linux-android${API_LEVEL}-clang"
export CXX_aarch64_linux_android="$TOOLCHAIN_BIN/aarch64-linux-android${API_LEVEL}-clang++"
export AR_aarch64_linux_android="$TOOLCHAIN_BIN/llvm-ar"

export CC_armv7_linux_androideabi="$TOOLCHAIN_BIN/armv7a-linux-androideabi${API_LEVEL}-clang"
export CXX_armv7_linux_androideabi="$TOOLCHAIN_BIN/armv7a-linux-androideabi${API_LEVEL}-clang++"
export AR_armv7_linux_androideabi="$TOOLCHAIN_BIN/llvm-ar"

export CC_x86_64_linux_android="$TOOLCHAIN_BIN/x86_64-linux-android${API_LEVEL}-clang"
export CXX_x86_64_linux_android="$TOOLCHAIN_BIN/x86_64-linux-android${API_LEVEL}-clang++"
export AR_x86_64_linux_android="$TOOLCHAIN_BIN/llvm-ar"

export CC_i686_linux_android="$TOOLCHAIN_BIN/i686-linux-android${API_LEVEL}-clang"
export CXX_i686_linux_android="$TOOLCHAIN_BIN/i686-linux-android${API_LEVEL}-clang++"
export AR_i686_linux_android="$TOOLCHAIN_BIN/llvm-ar"

# Configure cargo linkers via environment (alternative to config.toml)
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN_BIN/aarch64-linux-android${API_LEVEL}-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$TOOLCHAIN_BIN/armv7a-linux-androideabi${API_LEVEL}-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$TOOLCHAIN_BIN/x86_64-linux-android${API_LEVEL}-clang"
export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$TOOLCHAIN_BIN/i686-linux-android${API_LEVEL}-clang"

echo "Android build environment configured:"
echo "  ANDROID_SDK_HOME: $ANDROID_SDK_HOME"
echo "  ANDROID_NDK_HOME: $ANDROID_NDK_HOME"
echo "  Toolchain: $TOOLCHAIN_BIN"
echo "  API Level: $API_LEVEL"
echo "  Host: $HOST_TAG"
echo ""
echo "Available build commands:"
echo "  cargo build --target aarch64-linux-android -p blinc_platform_android   # ARM64 (most devices)"
echo "  cargo build --target armv7-linux-androideabi -p blinc_platform_android # ARMv7 (older 32-bit)"
echo "  cargo build --target x86_64-linux-android -p blinc_platform_android    # x86_64 (emulator)"
echo "  cargo build --target i686-linux-android -p blinc_platform_android      # x86 (old emulator)"
