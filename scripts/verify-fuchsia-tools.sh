#!/bin/bash
# Fuchsia Tools Verification Script for Blinc
#
# Verifies that all required Fuchsia development tools are installed
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
FUCHSIA_SDK="${FUCHSIA_SDK:-$HOME/.fuchsia/sdk-samples}"
FUCHSIA_BIN="${FUCHSIA_BIN:-$HOME/.fuchsia/bin}"

print_header() {
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  Fuchsia Tools Verification${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
}

check_tool() {
    local tool_name=$1
    local tool_path=$2
    local required=$3

    printf "%-20s" "$tool_name:"

    # Check direct path
    if [ -x "$tool_path" ] || [ -L "$tool_path" ]; then
        # For symlinks, just report found
        if [ -L "$tool_path" ]; then
            echo -e "${GREEN}OK${NC} (symlink -> bazel)"
            return 0
        fi
        local version=$("$tool_path" --version 2>/dev/null | head -1 || echo "found")
        echo -e "${GREEN}OK${NC} ($version)"
        return 0
    elif command -v "$tool_name" &> /dev/null; then
        local version=$("$tool_name" --version 2>/dev/null | head -1 || echo "found")
        echo -e "${GREEN}OK${NC} (in PATH: $version)"
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

check_rust_target() {
    local target=$1
    printf "%-30s" "$target:"

    if rustup target list --installed 2>/dev/null | grep -q "$target"; then
        echo -e "${GREEN}INSTALLED${NC}"
        return 0
    else
        echo -e "${YELLOW}NOT INSTALLED${NC}"
        return 1
    fi
}

verify_ffx() {
    echo -e "${BLUE}FFX Tool Status:${NC}"
    echo ""

    # Check FUCHSIA_BIN first (direct binary), then fall back to SDK tools wrapper
    local ffx_path="$FUCHSIA_BIN/ffx"
    if [ ! -x "$ffx_path" ] && [ ! -L "$ffx_path" ]; then
        ffx_path="$FUCHSIA_SDK/tools/ffx"
    fi
    if [ ! -x "$ffx_path" ] && [ ! -L "$ffx_path" ]; then
        ffx_path=$(command -v ffx 2>/dev/null || echo "")
    fi

    if [ -x "$ffx_path" ] || [ -L "$ffx_path" ]; then
        echo "Running ffx doctor (using $ffx_path)..."
        echo ""
        "$ffx_path" doctor 2>&1 | head -30 || true
    else
        echo -e "${RED}ffx not found${NC}"
        echo "Run ./scripts/setup-fuchsia-sdk.sh to install"
    fi
}

main() {
    print_header

    echo -e "${BLUE}SDK Location:${NC} $FUCHSIA_SDK"
    echo ""

    # Check core tools
    echo -e "${BLUE}Core Tools:${NC}"
    local failed=0

    check_tool "bazel" "$FUCHSIA_SDK/tools/bazel" "required" || ((failed++))
    check_tool "ffx" "$FUCHSIA_BIN/ffx" "required" || ((failed++))
    check_tool "shac" "$FUCHSIA_SDK/tools/shac" "optional"

    # SDK tools from the downloaded Bazel SDK
    echo ""
    echo -e "${BLUE}SDK Tools (via Bazel):${NC}"
    check_tool "fidlc" "$FUCHSIA_BIN/fidlc" "optional"
    check_tool "cmc" "$FUCHSIA_BIN/cmc" "optional"
    check_tool "far" "$FUCHSIA_BIN/far" "optional"
    check_tool "symbolizer" "$FUCHSIA_BIN/symbolizer" "optional"

    echo ""

    # Check Rust targets
    echo -e "${BLUE}Rust Targets:${NC}"
    if command -v rustup &> /dev/null; then
        check_rust_target "x86_64-unknown-fuchsia"
        check_rust_target "aarch64-unknown-fuchsia"
    else
        echo -e "${YELLOW}rustup not found, skipping Rust target check${NC}"
    fi

    echo ""

    # Environment check
    echo -e "${BLUE}Environment:${NC}"
    printf "%-20s" "FUCHSIA_SDK:"
    if [ -n "$FUCHSIA_SDK" ] && [ -d "$FUCHSIA_SDK" ]; then
        echo -e "${GREEN}SET${NC} ($FUCHSIA_SDK)"
    else
        echo -e "${YELLOW}NOT SET OR INVALID${NC}"
    fi

    printf "%-20s" "PATH includes SDK:"
    if echo "$PATH" | grep -q "$FUCHSIA_SDK/tools"; then
        echo -e "${GREEN}YES${NC}"
    else
        echo -e "${YELLOW}NO${NC} (add \$FUCHSIA_SDK/tools to PATH)"
    fi

    echo ""

    # FFX verification
    echo -e "${BLUE}FFX Doctor:${NC}"
    echo ""
    verify_ffx

    echo ""

    # Summary
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}All required tools are installed!${NC}"
        echo ""
        echo "You can now build Blinc for Fuchsia:"
        echo "  cargo build --features fuchsia --target x86_64-unknown-fuchsia"
    else
        echo -e "${RED}Some required tools are missing.${NC}"
        echo ""
        echo "Run the setup script to install:"
        echo "  ./scripts/setup-fuchsia-sdk.sh"
    fi
}

main "$@"
