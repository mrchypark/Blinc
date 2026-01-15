#!/bin/bash
# HarmonyOS / OpenHarmony SDK Setup Script for Blinc
#
# This script helps set up the development environment for building
# Blinc applications targeting HarmonyOS and OpenHarmony.
#
# References:
# - DevEco Studio: https://developer.huawei.com/consumer/en/deveco-studio/
# - OpenHarmony: https://www.openharmony.cn/
# - Servo OpenHarmony Guide: https://book.servo.org/building/openharmony.html

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
HARMONY_DIR="${HARMONY_DIR:-$HOME/.harmony}"
SDK_DIR="$HARMONY_DIR/sdk"
CMDLINE_TOOLS_DIR="$HARMONY_DIR/commandline-tools"

print_header() {
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  HarmonyOS SDK Setup for Blinc${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}[*]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

# Detect OS
detect_platform() {
    case "$(uname -s)" in
        Darwin)
            echo "macos"
            ;;
        Linux)
            echo "linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "windows"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
}

# Check if DevEco Studio is installed
check_deveco_studio() {
    print_step "Checking for DevEco Studio..."

    local platform=$(detect_platform)

    case "$platform" in
        macos)
            if [ -d "/Applications/DevEco Studio.app" ] || [ -d "$HOME/Applications/DevEco Studio.app" ]; then
                print_success "DevEco Studio found"
                return 0
            fi
            ;;
        windows)
            if [ -d "$LOCALAPPDATA/Huawei/DevEco Studio" ] || [ -d "$PROGRAMFILES/Huawei/DevEco Studio" ]; then
                print_success "DevEco Studio found"
                return 0
            fi
            ;;
        linux)
            print_warning "DevEco Studio is not available for Linux"
            print_info "You can use command-line tools instead"
            return 1
            ;;
    esac

    print_warning "DevEco Studio not found"
    return 1
}

# Check for command line tools
check_cmdline_tools() {
    print_step "Checking for HarmonyOS Command Line Tools..."

    if [ -d "$CMDLINE_TOOLS_DIR" ] && [ -x "$CMDLINE_TOOLS_DIR/bin/hvigorw" ]; then
        print_success "Command line tools found at $CMDLINE_TOOLS_DIR"
        return 0
    fi

    # Check if DEVECO_SDK_HOME is set
    if [ -n "$DEVECO_SDK_HOME" ] && [ -d "$DEVECO_SDK_HOME" ]; then
        print_success "DEVECO_SDK_HOME is set to: $DEVECO_SDK_HOME"
        return 0
    fi

    print_warning "Command line tools not found"
    return 1
}

# Check for HDC (Device Connector)
check_hdc() {
    print_step "Checking for HDC (Device Connector)..."

    if command -v hdc &> /dev/null; then
        local version=$(hdc version 2>/dev/null || echo "found")
        print_success "HDC found: $version"
        return 0
    fi

    if [ -x "$CMDLINE_TOOLS_DIR/sdk/*/toolchains/hdc" ]; then
        print_success "HDC found in SDK"
        return 0
    fi

    print_warning "HDC not found"
    return 1
}

# Check required dependencies
check_dependencies() {
    print_step "Checking dependencies..."

    local missing_deps=()

    # Check for Node.js
    if ! command -v node &> /dev/null; then
        missing_deps+=("nodejs")
    else
        local node_version=$(node --version | sed 's/v//')
        local major_version=$(echo "$node_version" | cut -d. -f1)
        if [ "$major_version" -lt 18 ]; then
            print_warning "Node.js version $node_version is old, HarmonyOS requires v18+"
        else
            print_success "Node.js $node_version found"
        fi
    fi

    # Check for Java
    if ! command -v java &> /dev/null; then
        missing_deps+=("java")
    else
        local java_version=$(java -version 2>&1 | head -1)
        print_success "Java found: $java_version"
    fi

    # Check for npm
    if ! command -v npm &> /dev/null; then
        missing_deps+=("npm")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_warning "Missing recommended dependencies: ${missing_deps[*]}"
        echo ""
        echo "Install Node.js 18+ and Java 17+ for HarmonyOS development"
    fi
}

# Print installation instructions
print_deveco_instructions() {
    local platform=$(detect_platform)

    echo ""
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  DevEco Studio Installation${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
    echo "DevEco Studio is the official IDE for HarmonyOS development."
    echo ""
    echo "Download from:"
    echo "  https://developer.huawei.com/consumer/en/deveco-studio/"
    echo ""

    case "$platform" in
        macos)
            echo "macOS Installation:"
            echo "  1. Download DevEco Studio for macOS"
            echo "  2. Open the .dmg file"
            echo "  3. Drag DevEco Studio to Applications"
            echo "  4. Open and complete the setup wizard"
            ;;
        windows)
            echo "Windows Installation:"
            echo "  1. Download DevEco Studio for Windows"
            echo "  2. Run the installer"
            echo "  3. Follow the setup wizard"
            echo "  4. Configure SDK path when prompted"
            ;;
        linux)
            echo "Linux (Command Line Tools):"
            echo "  DevEco Studio is not available for Linux."
            echo "  You can use Command Line Tools instead:"
            echo ""
            echo "  1. Create a Huawei Developer account"
            echo "  2. Download 'Command Line Tools for HarmonyOS NEXT' from:"
            echo "     https://developer.huawei.com/consumer/cn/deveco-studio/archive/"
            echo "  3. Extract to $CMDLINE_TOOLS_DIR"
            echo "  4. Run this script again to configure environment"
            ;;
    esac

    echo ""
    echo "Note: HarmonyOS NEXT tools may require a verified Huawei Developer account."
    echo ""
}

# Print OpenHarmony alternative
print_openharmony_instructions() {
    echo ""
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  OpenHarmony Alternative${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
    echo "OpenHarmony is the open-source version that doesn't require"
    echo "a Huawei account for basic development."
    echo ""
    echo "OpenHarmony SDK Download:"
    echo "  https://www.openharmony.cn/download"
    echo ""
    echo "Documentation:"
    echo "  https://book.servo.org/building/openharmony.html"
    echo ""
    echo "For Rust development, you may also need:"
    echo "  - ohos-sys crate for FFI bindings"
    echo "  - Custom target specification for aarch64-unknown-ohos"
    echo ""
}

# Setup environment variables
setup_environment() {
    print_step "Setting up environment variables..."

    local shell_rc=""

    case "$SHELL" in
        */zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        */bash)
            if [ -f "$HOME/.bash_profile" ]; then
                shell_rc="$HOME/.bash_profile"
            else
                shell_rc="$HOME/.bashrc"
            fi
            ;;
        *)
            shell_rc="$HOME/.profile"
            ;;
    esac

    # Check if already configured
    if grep -q "HARMONY_DIR=" "$shell_rc" 2>/dev/null; then
        print_warning "HarmonyOS environment already configured in $shell_rc"
        print_info "To update, remove the HarmonyOS section from $shell_rc and re-run"
        return 0
    fi

    # Find SDK platform path (written by download_ohos_sdk)
    local platform_path=""
    if [ -f "$HARMONY_DIR/.sdk_platform_path" ]; then
        platform_path=$(cat "$HARMONY_DIR/.sdk_platform_path")
    fi

    # Fallback to common locations
    if [ -z "$platform_path" ] || [ ! -d "$platform_path" ]; then
        for path in \
            "$SDK_DIR/sdk/packages/ohos-sdk/darwin" \
            "$SDK_DIR/sdk/packages/ohos-sdk/linux" \
            "$SDK_DIR/ohos-sdk/darwin" \
            "$SDK_DIR/ohos-sdk/linux" \
            "$CMDLINE_TOOLS_DIR/sdk"; do
            if [ -d "$path" ]; then
                platform_path="$path"
                break
            fi
        done
    fi

    if [ -z "$platform_path" ]; then
        print_warning "SDK platform path not found, using default"
        platform_path="$SDK_DIR"
    fi

    # Add environment setup
    cat >> "$shell_rc" << EOF

# HarmonyOS / OpenHarmony SDK (added by Blinc setup script)
export HARMONY_DIR="$HARMONY_DIR"
export OHOS_SDK_HOME="$platform_path"
export OHOS_NDK_HOME="\$OHOS_SDK_HOME/native"
export PATH="\$HARMONY_DIR/bin:\$OHOS_SDK_HOME/toolchains:\$OHOS_SDK_HOME/native/llvm/bin:\$PATH"
EOF

    print_success "Environment variables added to $shell_rc"
    print_warning "Run 'source $shell_rc' or start a new terminal to apply changes"
}

# Setup Rust target for OpenHarmony
setup_rust_ohos() {
    print_step "Setting up Rust for OpenHarmony..."

    if ! command -v rustup &> /dev/null; then
        print_warning "rustup not found, skipping Rust setup"
        return
    fi

    # OpenHarmony uses a custom target similar to Android
    # The official target triple is aarch64-unknown-linux-ohos or similar

    print_info "OpenHarmony Rust targets are experimental"
    print_info "You may need to use a custom target specification"

    # Create custom target JSON if needed
    local target_dir="$HARMONY_DIR/rust-targets"
    mkdir -p "$target_dir"

    # Create aarch64-unknown-linux-ohos target spec
    cat > "$target_dir/aarch64-unknown-linux-ohos.json" << 'EOF'
{
    "arch": "aarch64",
    "data-layout": "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
    "dynamic-linking": true,
    "env": "ohos",
    "executables": true,
    "has-rpath": true,
    "linker-flavor": "gcc",
    "llvm-target": "aarch64-unknown-linux-ohos",
    "max-atomic-width": 128,
    "os": "linux",
    "panic-strategy": "unwind",
    "position-independent-executables": true,
    "relro-level": "full",
    "supported-sanitizers": ["address", "memory", "thread", "hwaddress"],
    "target-c-int-width": "32",
    "target-endian": "little",
    "target-family": ["unix"],
    "target-mcount": "\u0001_mcount",
    "target-pointer-width": "64",
    "vendor": "unknown"
}
EOF

    print_success "Created custom target spec at $target_dir/aarch64-unknown-linux-ohos.json"
    print_info "Use with: cargo build --target $target_dir/aarch64-unknown-linux-ohos.json"
}

# Create directory structure
create_directories() {
    print_step "Creating directory structure..."

    mkdir -p "$HARMONY_DIR"
    mkdir -p "$SDK_DIR"
    mkdir -p "$CMDLINE_TOOLS_DIR"
    mkdir -p "$HARMONY_DIR/bin"

    print_success "Created directories at $HARMONY_DIR"
}

# Download OpenHarmony SDK from GitHub mirror
download_ohos_sdk() {
    print_step "Downloading OpenHarmony SDK..."

    local platform=$(detect_platform)
    local sdk_url=""
    local sdk_file=""

    # Select appropriate SDK based on platform
    case "$platform" in
        macos)
            # Check if running on Apple Silicon
            if [[ "$(uname -m)" == "arm64" ]]; then
                sdk_url="https://github.com/openharmony-rs/ohos-sdk/releases/download/v6.0.0.1/L2-SDK-MAC-M1-PUBLIC.tar.gz"
                sdk_file="L2-SDK-MAC-M1-PUBLIC.tar.gz"
            else
                sdk_url="https://github.com/openharmony-rs/ohos-sdk/releases/download/v6.0.0.1/ohos-sdk-mac-public.tar.gz"
                sdk_file="ohos-sdk-mac-public.tar.gz"
            fi
            ;;
        linux)
            print_warning "Linux SDK requires downloading split files"
            print_info "Download manually from: https://github.com/openharmony-rs/ohos-sdk/releases"
            return 1
            ;;
        *)
            print_error "Unsupported platform for SDK download"
            return 1
            ;;
    esac

    # Check if SDK already exists
    if [ -d "$SDK_DIR/native" ]; then
        print_success "OpenHarmony SDK already installed"
        return 0
    fi

    print_info "SDK URL: $sdk_url"
    print_warning "This download is ~1.2GB and may take a while..."

    local temp_dir=$(mktemp -d)
    local archive_path="$temp_dir/$sdk_file"

    # Download with progress
    if command -v curl &> /dev/null; then
        curl -L "$sdk_url" -o "$archive_path" --progress-bar || {
            print_error "Failed to download SDK"
            rm -rf "$temp_dir"
            return 1
        }
    elif command -v wget &> /dev/null; then
        wget "$sdk_url" -O "$archive_path" --show-progress || {
            print_error "Failed to download SDK"
            rm -rf "$temp_dir"
            return 1
        }
    else
        print_error "curl or wget required for download"
        rm -rf "$temp_dir"
        return 1
    fi

    # Extract SDK
    print_step "Extracting OpenHarmony SDK..."
    mkdir -p "$SDK_DIR"
    tar -xzf "$archive_path" -C "$SDK_DIR" || {
        print_error "Failed to extract SDK"
        rm -rf "$temp_dir"
        return 1
    }

    # Cleanup
    rm -rf "$temp_dir"

    # Find the ohos-sdk directory (different archive structures)
    local ohos_dir=""
    for path in "$SDK_DIR/sdk/packages/ohos-sdk" "$SDK_DIR/ohos-sdk" "$SDK_DIR"; do
        if [ -d "$path/darwin" ] || [ -d "$path/linux" ]; then
            ohos_dir="$path"
            break
        fi
    done

    if [ -z "$ohos_dir" ]; then
        print_warning "OHOS SDK directory not found after extraction"
        print_info "Check contents: find $SDK_DIR -name '*.zip' | head -5"
        return 1
    fi

    local platform_dir=""
    if [ -d "$ohos_dir/darwin" ]; then
        platform_dir="$ohos_dir/darwin"
    elif [ -d "$ohos_dir/linux" ]; then
        platform_dir="$ohos_dir/linux"
    fi

    if [ -z "$platform_dir" ]; then
        print_error "Platform SDK directory not found"
        return 1
    fi

    # Extract SDK component zips
    print_step "Extracting SDK components..."
    cd "$platform_dir"

    for zip_file in *.zip; do
        if [ -f "$zip_file" ]; then
            local component_name=$(echo "$zip_file" | sed 's/-darwin.*//; s/-linux.*//')
            if [ ! -d "$component_name" ]; then
                print_info "Extracting $zip_file..."
                unzip -q -o "$zip_file" || print_warning "Failed to extract $zip_file"
            fi
        fi
    done

    cd - > /dev/null

    # Verify native and toolchains directories
    local native_path="$platform_dir/native"
    local toolchains_path="$platform_dir/toolchains"

    if [ -d "$native_path" ]; then
        print_success "Native SDK at: $native_path"
    else
        print_warning "Native SDK not found (may need manual extraction)"
    fi

    if [ -d "$toolchains_path" ]; then
        print_success "Toolchains at: $toolchains_path"
    fi

    # Create symlinks to tools
    print_step "Creating tool symlinks..."
    mkdir -p "$HARMONY_DIR/bin"

    if [ -x "$toolchains_path/hdc" ]; then
        ln -sf "$toolchains_path/hdc" "$HARMONY_DIR/bin/hdc"
        print_success "Linked hdc"
    fi

    # Store the actual SDK path for environment setup
    echo "$platform_dir" > "$HARMONY_DIR/.sdk_platform_path"

    print_success "OpenHarmony SDK installed at $SDK_DIR"
    return 0
}

# Print verification commands
print_verification() {
    echo ""
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  Verification Commands${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
    echo "After installing DevEco Studio or Command Line Tools, verify with:"
    echo ""
    echo "  # Check hvigorw (build tool)"
    echo "  hvigorw --version"
    echo ""
    echo "  # Check HDC (device connector)"
    echo "  hdc version"
    echo ""
    echo "  # List connected devices"
    echo "  hdc list targets"
    echo ""
    echo "  # Build a HarmonyOS app"
    echo "  hvigorw assembleHap"
    echo ""
}

# Print summary
print_summary() {
    echo ""
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  Setup Summary${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
    echo "HarmonyOS development environment:"
    echo "  Directory:    $HARMONY_DIR"
    echo "  SDK Location: $SDK_DIR"
    echo ""
    echo "Next steps:"
    echo "  1. Install DevEco Studio (macOS/Windows) or Command Line Tools (Linux)"
    echo "  2. Run: source ~/.zshrc (or ~/.bashrc)"
    echo "  3. Run: ./scripts/verify-harmony-tools.sh"
    echo ""
    echo "Build Blinc for HarmonyOS:"
    echo "  cargo build --features harmony"
    echo ""
}

# Main installation flow
main() {
    print_header

    local platform=$(detect_platform)
    print_step "Detected platform: $platform"
    echo ""

    # Check dependencies
    check_dependencies
    echo ""

    # Check existing installations
    local has_deveco=false
    local has_cmdline=false

    check_deveco_studio && has_deveco=true || true
    check_cmdline_tools && has_cmdline=true || true
    check_hdc || true
    echo ""

    # Create directories
    create_directories

    # If nothing installed, try to download OpenHarmony SDK
    if [ "$has_deveco" = false ] && [ "$has_cmdline" = false ]; then
        echo ""
        print_step "No DevEco Studio or Command Line Tools found"
        print_info "Attempting to download OpenHarmony SDK from GitHub mirror..."
        echo ""

        if download_ohos_sdk; then
            print_success "OpenHarmony SDK downloaded successfully!"
        else
            print_warning "SDK download failed or was skipped"
            print_deveco_instructions
            print_openharmony_instructions
        fi
    fi

    # Setup environment
    setup_environment
    echo ""

    # Setup Rust
    setup_rust_ohos
    echo ""

    # Print verification
    print_verification

    # Print summary
    print_summary
}

# Run main
main "$@"
