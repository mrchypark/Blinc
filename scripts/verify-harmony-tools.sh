#!/bin/bash
# HarmonyOS Tools Verification Script for Blinc
#
# Verifies that all required HarmonyOS/OpenHarmony development tools are installed
# and properly configured for Blinc development.

# Don't exit on first error - we want to show all checks
# set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
HARMONY_DIR="${HARMONY_DIR:-$HOME/.harmony}"
OHOS_SDK_HOME="${OHOS_SDK_HOME:-$HARMONY_DIR/sdk/sdk/packages/ohos-sdk/darwin}"
HARMONY_BIN="${HARMONY_DIR}/bin"

print_header() {
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  HarmonyOS Tools Verification${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
}

check_tool() {
    local tool_name=$1
    local required=$2

    printf "%-20s" "$tool_name:"

    if command -v "$tool_name" &> /dev/null; then
        local version=$("$tool_name" --version 2>/dev/null | head -1 || "$tool_name" version 2>/dev/null | head -1 || echo "found")
        echo -e "${GREEN}OK${NC} ($version)"
        return 0
    else
        if [ "$required" = "required" ]; then
            echo -e "${RED}MISSING${NC}"
            return 1
        else
            echo -e "${YELLOW}OPTIONAL${NC} (not found)"
            return 0
        fi
    fi
}

check_env_var() {
    local var_name=$1
    local var_value="${!var_name}"

    printf "%-20s" "$var_name:"

    if [ -n "$var_value" ]; then
        if [ -d "$var_value" ]; then
            echo -e "${GREEN}SET${NC} ($var_value)"
            return 0
        else
            echo -e "${YELLOW}SET BUT INVALID${NC} (path doesn't exist: $var_value)"
            return 1
        fi
    else
        echo -e "${YELLOW}NOT SET${NC}"
        return 1
    fi
}

check_deveco_studio() {
    printf "%-20s" "DevEco Studio:"

    case "$(uname -s)" in
        Darwin)
            if [ -d "/Applications/DevEco Studio.app" ]; then
                echo -e "${GREEN}INSTALLED${NC} (/Applications)"
                return 0
            elif [ -d "$HOME/Applications/DevEco Studio.app" ]; then
                echo -e "${GREEN}INSTALLED${NC} (~/Applications)"
                return 0
            fi
            ;;
        Linux)
            echo -e "${YELLOW}N/A${NC} (not available on Linux)"
            return 0
            ;;
        MINGW*|MSYS*|CYGWIN*)
            if [ -d "$LOCALAPPDATA/Huawei/DevEco Studio" ]; then
                echo -e "${GREEN}INSTALLED${NC}"
                return 0
            fi
            ;;
    esac

    echo -e "${YELLOW}NOT FOUND${NC}"
    return 1
}

check_ndk() {
    printf "%-20s" "OpenHarmony NDK:"

    local ndk_path="${OHOS_NDK_HOME:-$OHOS_SDK_HOME/native}"

    if [ -d "$ndk_path" ]; then
        echo -e "${GREEN}FOUND${NC} ($ndk_path)"
        # Check for clang
        if [ -x "$ndk_path/llvm/bin/clang" ]; then
            local clang_version=$("$ndk_path/llvm/bin/clang" --version 2>/dev/null | head -1)
            echo -e "  ${GREEN}clang:${NC} $clang_version"
        fi
        return 0
    else
        echo -e "${YELLOW}NOT FOUND${NC}"
        return 1
    fi
}

check_hdc_binary() {
    printf "%-20s" "HDC (binary):"

    local hdc_path="$HARMONY_BIN/hdc"
    if [ ! -x "$hdc_path" ]; then
        hdc_path="$OHOS_SDK_HOME/toolchains/hdc"
    fi

    if [ -x "$hdc_path" ]; then
        local version=$("$hdc_path" version 2>/dev/null || echo "found")
        echo -e "${GREEN}OK${NC} ($version)"
        return 0
    else
        echo -e "${YELLOW}NOT FOUND${NC}"
        return 1
    fi
}

check_device_connectivity() {
    echo -e "${BLUE}Device Connectivity:${NC}"
    echo ""

    local hdc_cmd=""
    if command -v hdc &> /dev/null; then
        hdc_cmd="hdc"
    elif [ -x "$HARMONY_BIN/hdc" ]; then
        hdc_cmd="$HARMONY_BIN/hdc"
    elif [ -x "$OHOS_SDK_HOME/toolchains/hdc" ]; then
        hdc_cmd="$OHOS_SDK_HOME/toolchains/hdc"
    fi

    if [ -n "$hdc_cmd" ]; then
        echo "Running hdc list targets..."
        "$hdc_cmd" list targets 2>&1 || echo "(no devices connected)"
    else
        echo -e "${YELLOW}HDC not available${NC}"
    fi
}

main() {
    print_header

    # Environment variables
    echo -e "${BLUE}Environment Variables:${NC}"
    check_env_var "HARMONY_DIR"
    check_env_var "OHOS_SDK_HOME"
    check_env_var "OHOS_NDK_HOME"
    echo ""

    # DevEco Studio
    echo -e "${BLUE}IDE:${NC}"
    check_deveco_studio
    echo ""

    # Build tools (from DevEco or command line tools)
    echo -e "${BLUE}Build Tools:${NC}"
    local failed=0
    check_tool "hvigorw" "optional" || ((failed++))
    check_tool "ohpm" "optional"
    echo ""

    # Device tools
    echo -e "${BLUE}Device Tools:${NC}"
    check_tool "hdc" "optional" || check_hdc_binary || ((failed++))
    echo ""

    # NDK
    echo -e "${BLUE}Native Development:${NC}"
    check_ndk || ((failed++))
    echo ""

    # Dependencies
    echo -e "${BLUE}Dependencies:${NC}"
    check_tool "node" "optional"
    check_tool "java" "optional"
    check_tool "npm" "optional"
    echo ""

    # Device connectivity (only if hdc available)
    if command -v hdc &> /dev/null || [ -x "$HARMONY_BIN/hdc" ] || [ -x "$OHOS_SDK_HOME/toolchains/hdc" ]; then
        check_device_connectivity
        echo ""
    fi

    # Summary
    echo -e "${BLUE}======================================${NC}"
    local hdc_available=false
    local ndk_available=false

    if command -v hdc &> /dev/null || [ -x "$HARMONY_BIN/hdc" ] || [ -x "$OHOS_SDK_HOME/toolchains/hdc" ]; then
        hdc_available=true
    fi
    if [ -d "$OHOS_SDK_HOME/native" ] || [ -d "${OHOS_NDK_HOME:-/nonexistent}" ]; then
        ndk_available=true
    fi

    if [ "$hdc_available" = true ] && [ "$ndk_available" = true ]; then
        echo -e "${GREEN}OpenHarmony SDK is installed!${NC}"
        echo ""
        echo "Build Blinc for HarmonyOS:"
        echo "  cargo build --features harmony"
        echo ""
        echo "Use HDC to connect devices:"
        echo "  hdc list targets"
        echo "  hdc install <your-app>.hap"
    elif [ "$ndk_available" = true ]; then
        echo -e "${YELLOW}OpenHarmony NDK installed (HDC not found)${NC}"
        echo ""
        echo "You can build native code but cannot deploy to devices."
        echo "Install DevEco Studio for full toolchain."
    else
        echo -e "${YELLOW}OpenHarmony SDK not fully configured${NC}"
        echo ""
        echo "Run: ./scripts/setup-harmony-sdk.sh"
    fi
}

main "$@"
