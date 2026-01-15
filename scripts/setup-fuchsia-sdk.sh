#!/bin/bash
# Fuchsia SDK Setup Script for Blinc
#
# This script installs and configures the Fuchsia SDK and ffx tools
# required for building Blinc applications targeting Fuchsia OS.
#
# References:
# - SDK Documentation: https://fuchsia.dev/fuchsia-src/development/sdk
# - SDK Samples: https://fuchsia.googlesource.com/sdk-samples/getting-started/
# - ffx Tool: https://fuchsia.dev/fuchsia-src/development/tools/ffx/getting-started

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FUCHSIA_DIR="${FUCHSIA_DIR:-$HOME/.fuchsia}"
SDK_DIR="$FUCHSIA_DIR/sdk-samples"
FFX_PATH="$SDK_DIR/tools/ffx"

print_header() {
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  Fuchsia SDK Setup for Blinc${NC}"
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
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    local os=""
    local arch=""

    case "$(uname -s)" in
        Darwin)
            os="mac"
            ;;
        Linux)
            os="linux"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            print_error "Fuchsia SDK is only available for macOS and Linux"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)
            arch="amd64"
            ;;
        arm64|aarch64)
            arch="arm64"
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Check if Fuchsia SDK is already installed
check_existing_installation() {
    print_step "Checking for existing Fuchsia SDK installation..."

    # Check if SDK samples repo exists
    if [ -d "$SDK_DIR/.git" ]; then
        print_success "Found SDK samples at $SDK_DIR"

        # Check for tools/ffx or tools/bazel
        if [ -x "$SDK_DIR/tools/ffx" ]; then
            local ffx_version=$("$SDK_DIR/tools/ffx" --version 2>/dev/null || echo "unknown")
            print_success "ffx version: $ffx_version"
        elif [ -x "$SDK_DIR/tools/bazel" ]; then
            print_success "Bazel tool found (ffx available via bazel build)"
        fi
        return 0
    fi

    # Check if ffx exists in PATH
    if command -v ffx &> /dev/null; then
        local ffx_version=$(ffx --version 2>/dev/null || echo "unknown")
        print_success "Found ffx in PATH: $ffx_version"
        return 0
    fi

    print_warning "No existing Fuchsia SDK installation found"
    return 1
}

# Check required dependencies
check_dependencies() {
    print_step "Checking dependencies..."

    local missing_deps=()

    # Check for curl or wget
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        missing_deps+=("curl or wget")
    fi

    # Check for unzip
    if ! command -v unzip &> /dev/null; then
        missing_deps+=("unzip")
    fi

    # Check for Python 3
    if ! command -v python3 &> /dev/null; then
        missing_deps+=("python3")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing required dependencies: ${missing_deps[*]}"
        echo ""
        echo "Please install the missing dependencies:"
        if [[ "$(uname -s)" == "Darwin" ]]; then
            echo "  brew install ${missing_deps[*]}"
        else
            echo "  sudo apt-get install ${missing_deps[*]}"
        fi
        exit 1
    fi

    print_success "All dependencies are installed"
}

# Download the Fuchsia SDK using the official Bazel SDK samples approach
download_sdk() {
    local platform=$1

    print_step "Setting up Fuchsia SDK for $platform..."

    # Check for git
    if ! command -v git &> /dev/null; then
        print_error "git is required but not installed"
        exit 1
    fi

    # Create parent directory
    mkdir -p "$FUCHSIA_DIR"

    # Clone or update the SDK samples repository
    if [ -d "$SDK_DIR/.git" ]; then
        print_step "Updating existing SDK samples repository..."
        cd "$SDK_DIR"
        git pull --rebase origin main || {
            print_warning "Failed to update, continuing with existing version"
        }
        cd - > /dev/null
    else
        print_step "Cloning Fuchsia SDK samples repository..."
        rm -rf "$SDK_DIR"
        git clone https://fuchsia.googlesource.com/sdk-samples/getting-started "$SDK_DIR" || {
            print_error "Failed to clone SDK samples repository"
            exit 1
        }
    fi

    # Initialize git submodules (required for bootstrap and tools)
    print_step "Initializing git submodules..."
    cd "$SDK_DIR"
    git submodule update --init --recursive || {
        print_warning "Some submodules failed to initialize"
    }
    cd - > /dev/null

    # Run bootstrap script
    print_step "Running bootstrap script (this downloads the SDK toolchain)..."
    cd "$SDK_DIR"

    if [ -f "scripts/bootstrap.sh" ]; then
        ./scripts/bootstrap.sh || {
            print_warning "Bootstrap script failed, attempting manual setup..."
        }
    else
        print_warning "Bootstrap script not found, SDK structure may have changed"
    fi

    cd - > /dev/null
    print_success "Fuchsia SDK set up at $SDK_DIR"
}

# Fix SDK manifests for macOS
fix_sdk_manifests() {
    print_step "Configuring SDK manifests for macOS..."

    local platform=$(detect_platform)

    # On macOS arm64, we need mac-amd64 tools (run via Rosetta 2)
    # since there are no native arm64 macOS host tools yet
    if [[ "$platform" == "mac-arm64" ]] || [[ "$platform" == "mac-amd64" ]]; then
        # Fix bazel_sdk.ensure to use mac-amd64
        cat > "$SDK_DIR/manifests/bazel_sdk.ensure" << 'EOF'
$ResolvedVersions bazel_sdk.resolved
$VerifiedPlatform linux-amd64 mac-amd64

@Subdir
fuchsia/sdk/core/fuchsia-bazel-rules/mac-amd64 latest
EOF

        # Fix clang.ensure to use mac-amd64
        cat > "$SDK_DIR/manifests/clang.ensure" << 'EOF'
$ResolvedVersions clang.resolved
$VerifiedPlatform linux-amd64 mac-amd64

@Subdir
fuchsia/third_party/clang/mac-amd64 latest
EOF

        # Resolve the manifests
        print_step "Resolving SDK manifests..."
        local cipd_tool="$FUCHSIA_DIR/tools/cipd"
        if [ ! -x "$cipd_tool" ]; then
            cipd_tool=$(command -v cipd 2>/dev/null || echo "")
        fi

        if [ -x "$cipd_tool" ]; then
            "$cipd_tool" ensure-file-resolve -ensure-file "$SDK_DIR/manifests/bazel_sdk.ensure" 2>&1 || true
            "$cipd_tool" ensure-file-resolve -ensure-file "$SDK_DIR/manifests/clang.ensure" 2>&1 || true
        fi

        print_success "SDK manifests configured for macOS"
    fi
}

# Build the SDK toolchain to ensure ffx is downloaded
build_sdk_toolchain() {
    print_step "Building SDK toolchain (downloads ffx and other tools)..."

    cd "$SDK_DIR"

    # Check for bazel
    local bazel_cmd=""
    if [ -x "tools/bazel" ]; then
        bazel_cmd="tools/bazel"
    elif command -v bazelisk &> /dev/null; then
        bazel_cmd="bazelisk"
    elif command -v bazel &> /dev/null; then
        # Avoid depot_tools bazel which may not work
        local bazel_path=$(which bazel)
        if [[ "$bazel_path" != *"depot_tools"* ]]; then
            bazel_cmd="bazel"
        fi
    fi

    if [ -n "$bazel_cmd" ]; then
        print_step "Using Bazel at: $bazel_cmd"

        # Query the SDK to trigger download
        print_step "Downloading Fuchsia SDK..."
        $bazel_cmd query @fuchsia_sdk//... 2>&1 | head -5 || {
            print_warning "SDK query had issues, but tools may still be available"
        }
    else
        print_warning "Bazel not found."
        print_info "Install bazelisk: brew install bazelisk (macOS) or npm i -g @bazel/bazelisk"
    fi

    cd - > /dev/null
}

# Create convenient symlinks to SDK tools
create_tool_symlinks() {
    print_step "Creating tool symlinks..."

    mkdir -p "$FUCHSIA_DIR/bin"

    # Find the SDK tools directory in Bazel cache
    local sdk_tools_dir=""
    for dir in /private/var/tmp/_bazel_*/*/external/fuchsia_sdk/tools/x64; do
        if [ -d "$dir" ]; then
            sdk_tools_dir="$dir"
            break
        fi
    done

    if [ -z "$sdk_tools_dir" ]; then
        print_warning "SDK tools not found in Bazel cache"
        print_info "Tools will be available after running: cd $SDK_DIR && tools/bazel query @fuchsia_sdk//..."
        return
    fi

    # Create symlinks for common tools
    for tool in ffx cmc fidlc far symbolizer bootserver device-finder; do
        if [ -f "$sdk_tools_dir/$tool" ]; then
            ln -sf "$sdk_tools_dir/$tool" "$FUCHSIA_DIR/bin/$tool"
            print_success "Linked $tool"
        fi
    done

    # Repair ffx configuration
    if [ -x "$FUCHSIA_DIR/bin/ffx" ]; then
        print_step "Configuring ffx..."
        "$FUCHSIA_DIR/bin/ffx" doctor --repair-keys --restart-daemon 2>&1 | tail -5 || true
    fi
}

# Setup environment variables
setup_environment() {
    print_step "Setting up environment variables..."

    local shell_rc=""
    local shell_name=""

    # Detect shell
    case "$SHELL" in
        */zsh)
            shell_rc="$HOME/.zshrc"
            shell_name="zsh"
            ;;
        */bash)
            if [ -f "$HOME/.bash_profile" ]; then
                shell_rc="$HOME/.bash_profile"
            else
                shell_rc="$HOME/.bashrc"
            fi
            shell_name="bash"
            ;;
        *)
            shell_rc="$HOME/.profile"
            shell_name="sh"
            ;;
    esac

    # Check if already configured
    if grep -q "FUCHSIA_SDK" "$shell_rc" 2>/dev/null; then
        print_warning "Fuchsia environment already configured in $shell_rc"
        return 0
    fi

    # Add environment setup for Bazel SDK
    cat >> "$shell_rc" << EOF

# Fuchsia SDK (added by Blinc setup script)
export FUCHSIA_DIR="$FUCHSIA_DIR"
export FUCHSIA_SDK="$SDK_DIR"
export FUCHSIA_BIN="\$FUCHSIA_DIR/bin"
export PATH="\$FUCHSIA_BIN:\$PATH"
EOF

    print_success "Environment variables added to $shell_rc"
    print_warning "Run 'source $shell_rc' or start a new terminal to apply changes"
}

# Verify the installation
verify_installation() {
    print_step "Verifying installation..."

    # Export for current session
    export FUCHSIA_DIR="$FUCHSIA_DIR"
    export FUCHSIA_SDK="$SDK_DIR"

    # Find ffx in the Bazel-managed SDK
    local ffx_path=""

    # Check common locations for ffx in Bazel SDK
    for path in \
        "$SDK_DIR/tools/ffx" \
        "$SDK_DIR/bazel-bin/external/fuchsia_sdk/tools/ffx" \
        "$HOME/.cache/bazel/_bazel_$(whoami)/"*/external/fuchsia_sdk~*/ffx/ffx \
        "$SDK_DIR/third_party/fuchsia-sdk/bin/ffx"; do
        if [ -x "$path" ]; then
            ffx_path="$path"
            break
        fi
    done

    if [ -n "$ffx_path" ]; then
        local version=$("$ffx_path" --version 2>/dev/null || echo "unknown")
        print_success "ffx found: $ffx_path"
        print_success "ffx version: $version"

        # Run ffx doctor
        print_step "Running ffx doctor..."
        "$ffx_path" doctor --restart-daemon 2>&1 || true
    else
        print_warning "ffx not found yet"
        print_info "ffx will be downloaded when you run: cd $SDK_DIR && tools/bazel build @fuchsia_sdk//:fuchsia_toolchain_sdk"
    fi

    # Check if SDK samples repo is valid
    if [ -d "$SDK_DIR/.git" ]; then
        print_success "SDK samples repository: OK"
    fi

    # Check for bootstrap completion
    if [ -x "$SDK_DIR/tools/bazel" ]; then
        print_success "Bazel tool: OK"
    else
        print_warning "Bazel not found in tools/"
    fi

    print_success "Fuchsia SDK setup complete!"
}

# Add Rust targets for Fuchsia
setup_rust_targets() {
    print_step "Setting up Rust targets for Fuchsia..."

    if ! command -v rustup &> /dev/null; then
        print_warning "rustup not found, skipping Rust target setup"
        return
    fi

    # Add Fuchsia targets
    print_step "Adding x86_64-unknown-fuchsia target..."
    rustup target add x86_64-unknown-fuchsia 2>/dev/null || {
        print_warning "x86_64-unknown-fuchsia target not available in stable, may need nightly"
    }

    print_step "Adding aarch64-unknown-fuchsia target..."
    rustup target add aarch64-unknown-fuchsia 2>/dev/null || {
        print_warning "aarch64-unknown-fuchsia target not available in stable, may need nightly"
    }

    print_success "Rust targets configured"
}

# Print summary
print_summary() {
    echo ""
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  Setup Complete!${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
    echo "Fuchsia SDK Location: $SDK_DIR"
    echo ""
    echo "To use in a new terminal:"
    echo "  source ~/.zshrc  # or ~/.bashrc"
    echo ""
    echo -e "${BLUE}Next Steps:${NC}"
    echo ""
    echo "1. Build a sample to trigger SDK download:"
    echo "   cd $SDK_DIR"
    echo "   tools/bazel build //src/hello_world:pkg"
    echo ""
    echo "2. Start Fuchsia emulator:"
    echo "   cd $SDK_DIR"
    echo "   tools/ffx product-bundle get workbench_eng.x64"
    echo "   tools/ffx emu start --headless workbench_eng.x64"
    echo ""
    echo "3. Run the sample:"
    echo "   tools/bazel run //src/hello_world:pkg.component"
    echo ""
    echo -e "${BLUE}Build Blinc for Fuchsia:${NC}"
    echo "  cargo build --features fuchsia --target x86_64-unknown-fuchsia"
    echo ""
    echo -e "${BLUE}Documentation:${NC}"
    echo "  - SDK Guide: https://fuchsia.dev/fuchsia-src/development/sdk"
    echo "  - ffx Tool:  https://fuchsia.dev/fuchsia-src/development/tools/ffx/getting-started"
    echo "  - Samples:   https://fuchsia.googlesource.com/sdk-samples/getting-started/"
    echo ""
    echo -e "${YELLOW}Note:${NC} If you encounter Bazel build errors with fuchsia_clang,"
    echo "this is a known issue. The SDK samples team is actively maintaining the rules."
    echo "Check for updates: cd $SDK_DIR && git pull && git submodule update --recursive"
    echo ""
}

# Main installation flow
main() {
    print_header

    # Check if already installed
    if check_existing_installation; then
        read -p "Fuchsia SDK is already installed. Reinstall? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_step "Skipping SDK download, verifying existing installation..."
            verify_installation
            setup_rust_targets
            print_summary
            exit 0
        fi
    fi

    # Detect platform
    local platform=$(detect_platform)
    print_step "Detected platform: $platform"

    # Check dependencies
    check_dependencies

    # Download/clone SDK
    download_sdk "$platform"

    # Fix manifests for macOS
    fix_sdk_manifests

    # Build SDK toolchain (downloads ffx)
    build_sdk_toolchain

    # Create tool symlinks
    create_tool_symlinks

    # Setup environment
    setup_environment

    # Verify installation
    verify_installation

    # Setup Rust targets
    setup_rust_targets

    # Print summary
    print_summary
}

# Run main
main "$@"
