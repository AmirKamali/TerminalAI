#!/bin/bash

# Cross-platform build script for Terminal AI

set -e

# Parse command line arguments
ARCH="all"
PLATFORM="all"
HELP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --arch=*|--architecture=*)
            ARCH="${1#*=}"
            shift
            ;;
        --platform=*|--os=*)
            PLATFORM="${1#*=}"
            shift
            ;;
        --target=*)
            TARGET_OVERRIDE="${1#*=}"
            shift
            ;;
        -h|--help)
            HELP=true
            shift
            ;;
        *)
            echo "Unknown option $1"
            HELP=true
            shift
            ;;
    esac
done

if [ "$HELP" = true ]; then
    echo "Terminal AI Build Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --arch=ARCH        Architecture: arm64, x86_64, all (default: all)"
    echo "  --platform=OS      Platform: macos, linux, all (default: all)"
    echo "  --target=TARGET    Specific Rust target (overrides arch/platform)"
    echo "  -h, --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                           # Build for all platforms"
    echo "  $0 --arch=arm64             # Build ARM64 for all platforms"
    echo "  $0 --platform=macos         # Build all architectures for macOS"
    echo "  $0 --arch=arm64 --platform=macos  # Build ARM64 for macOS only"
    echo "  $0 --target=aarch64-apple-darwin  # Build specific Rust target"
    exit 0
fi

echo "🚀 Building Terminal AI..."
echo "📋 Build configuration:"
echo "   Architecture: $ARCH"
echo "   Platform: $PLATFORM"
if [ -n "$TARGET_OVERRIDE" ]; then
    echo "   Target override: $TARGET_OVERRIDE"
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Detect current platform
CURRENT_OS=$(uname -s)
CURRENT_ARCH=$(uname -m)

echo "🔍 Current platform: $CURRENT_OS ($CURRENT_ARCH)"

# Define target platforms
ALL_TARGETS=(
    "x86_64-apple-darwin"     # macOS Intel
    "aarch64-apple-darwin"    # macOS Apple Silicon
    "x86_64-unknown-linux-gnu" # Linux x86_64
    "aarch64-unknown-linux-gnu" # Linux ARM64
)

# Filter targets based on arguments
TARGETS=()

if [ -n "$TARGET_OVERRIDE" ]; then
    TARGETS=("$TARGET_OVERRIDE")
else
    for target in "${ALL_TARGETS[@]}"; do
        # Check architecture filter
        arch_match=false
        if [ "$ARCH" = "all" ]; then
            arch_match=true
        elif [ "$ARCH" = "arm64" ] && [[ $target == *"aarch64"* ]]; then
            arch_match=true
        elif [ "$ARCH" = "x86_64" ] && [[ $target == *"x86_64"* ]]; then
            arch_match=true
        fi
        
        # Check platform filter
        platform_match=false
        if [ "$PLATFORM" = "all" ]; then
            platform_match=true
        elif [ "$PLATFORM" = "macos" ] && [[ $target == *"apple-darwin"* ]]; then
            platform_match=true
        elif [ "$PLATFORM" = "linux" ] && [[ $target == *"linux-gnu"* ]]; then
            platform_match=true
        fi
        
        # Add target if both filters match
        if [ "$arch_match" = true ] && [ "$platform_match" = true ]; then
            TARGETS+=("$target")
        fi
    done
fi

echo "🎯 Selected targets: ${TARGETS[*]}"

# Create output directories
echo "📁 Creating output directories..."
mkdir -p release/macos-arm64
mkdir -p release/macos-x86_64
mkdir -p release/linux-arm64
mkdir -p release/linux-x86_64

# Function to install dependencies for cross-compilation
install_cross_deps() {
    local target=$1
    
    # Check if we're on Ubuntu/Debian and building for ARM64 Linux
    if [[ "$target" == "aarch64-unknown-linux-gnu" ]] && command -v apt-get &> /dev/null; then
        echo "📦 Installing cross-compilation dependencies for ARM64 Linux..."
        
        # Check if we're in a CI environment (GitHub Actions)
        if [[ -n "$CI" ]] || [[ -n "$GITHUB_ACTIONS" ]]; then
            echo "🔧 CI environment detected - using CI-specific installation..."
            sudo apt-get update -qq
            sudo apt-get install -y \
                gcc-aarch64-linux-gnu \
                g++-aarch64-linux-gnu \
                libssl-dev:arm64 \
                pkg-config \
                crossbuild-essential-arm64
        else
            echo "🔧 Local environment - checking if dependencies are already installed..."
            # Check if dependencies are already installed
            if ! dpkg -l | grep -q "gcc-aarch64-linux-gnu"; then
                sudo apt-get update -qq
                sudo apt-get install -y \
                    gcc-aarch64-linux-gnu \
                    g++-aarch64-linux-gnu \
                    libssl-dev:arm64 \
                    pkg-config \
                    crossbuild-essential-arm64
            else
                echo "✅ Cross-compilation dependencies already installed"
            fi
        fi
        
        echo "✅ Cross-compilation dependencies installed"
    fi
}

# Function to build for a specific target
build_target() {
    local target=$1
    local platform_name=$2
    local output_dir=$3
    
    echo "🔨 Building for $platform_name ($target)..."
    
    # Install cross-compilation dependencies if needed
    install_cross_deps "$target"
    
    # Add target if not already installed
    rustup target add $target 2>/dev/null || true
    
    # Set up environment for cross-compilation
    if [[ "$target" == "aarch64-unknown-linux-gnu" ]]; then
        echo "🔧 Setting up cross-compilation environment for ARM64..."
        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/pkgconfig"
        export OPENSSL_DIR="/usr"
        export OPENSSL_LIB_DIR="/usr/lib/aarch64-linux-gnu"
        export OPENSSL_INCLUDE_DIR="/usr/include"
        export CC_aarch64_unknown_linux_gnu="aarch64-linux-gnu-gcc"
        export CXX_aarch64_unknown_linux_gnu="aarch64-linux-gnu-g++"
        
        # Debug: Check what's actually available
        echo "🔍 Debug: Checking OpenSSL installation..."
        echo "PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
        ls -la /usr/lib/aarch64-linux-gnu/pkgconfig/ | grep openssl || echo "No openssl.pc in ARM64 pkgconfig"
        ls -la /usr/lib/pkgconfig/ | grep openssl || echo "No openssl.pc in system pkgconfig"
        pkg-config --list-all | grep openssl || echo "No openssl found by pkg-config"
    fi
    
    # Build for target
    if cargo build --release --target $target; then
        echo "✅ Build successful for $platform_name"
        
        # Copy binaries to appropriate directory
        local target_dir="target/$target/release"
        local bin_extension=""
        
        # Add .exe extension for Windows (future proofing)
        if [[ $target == *"windows"* ]]; then
            bin_extension=".exe"
        fi
        
            if [ -f "$target_dir/tai$bin_extension" ]; then
        cp "$target_dir/tai$bin_extension" "$output_dir/"
            cp "$target_dir/cp_ai$bin_extension" "$output_dir/"
            cp "$target_dir/grep_ai$bin_extension" "$output_dir/"
            # Copy find_ai if it exists
            if [ -f "$target_dir/find_ai$bin_extension" ]; then
                cp "$target_dir/find_ai$bin_extension" "$output_dir/"
            fi
            
            # Copy config file to output directory
            if [ -f "terminalai.conf" ]; then
                cp "terminalai.conf" "$output_dir/"
                echo "  📦 Copied binaries and config to $output_dir/"
            else
                echo "  📦 Copied binaries to $output_dir/"
                echo "  ⚠️  Warning: terminalai.conf not found in root directory"
            fi
        fi
    else
        echo "⚠️  Build failed for $platform_name - skipping..."
    fi
}

# Build for each target platform
for target in "${TARGETS[@]}"; do
    case $target in
        "x86_64-apple-darwin")
            if [[ "$CURRENT_OS" == "Darwin" ]]; then
                build_target "$target" "macOS x86_64" "release/macos-x86_64"
            else
                echo "⚠️  Skipping macOS x86_64 target - requires macOS host"
            fi
            ;;
        "aarch64-apple-darwin")
            if [[ "$CURRENT_OS" == "Darwin" ]]; then
                build_target "$target" "macOS ARM64" "release/macos-arm64"
            else
                echo "⚠️  Skipping macOS ARM64 target - requires macOS host"
            fi
            ;;
        "x86_64-unknown-linux-gnu")
            build_target "$target" "Linux x86_64" "release/linux-x86_64"
            ;;
        "aarch64-unknown-linux-gnu")
            build_target "$target" "Linux ARM64" "release/linux-arm64"
            ;;
        *)
            # Handle custom targets
            if [[ $target == *"apple-darwin"* ]]; then
                if [[ "$CURRENT_OS" == "Darwin" ]]; then
                    build_target "$target" "macOS Custom" "release/macos-custom"
                else
                    echo "⚠️  Skipping macOS target $target - requires macOS host"
                fi
            else
                build_target "$target" "Custom" "release/custom"
            fi
            ;;
    esac
done

echo ""
echo "✅ Build complete!"
echo ""
echo "📦 Binaries created:"

# Check each possible output directory
for dir in release/macos-arm64 release/macos-x86_64 release/linux-arm64 release/linux-x86_64 release/macos-custom release/custom; do
    if [ -d "$dir" ] && [ "$(ls -A "$dir" 2>/dev/null)" ]; then
        echo "  $dir:"
        ls -la "$dir/" | tail -n +2 | awk '{print "    - " $9 " (" $5 " bytes)"}'
    fi
done

echo ""
echo "🚀 Installation instructions:"
echo ""
echo "For macOS:"
echo "  sudo cp release/macos-*/* /usr/local/bin/"
echo "  or: cp release/macos-*/* ~/.local/bin/"
echo ""
echo "For Linux:"
echo "  sudo cp release/linux-*/* /usr/local/bin/"
echo "  or: cp release/linux-*/* ~/.local/bin/"
echo ""
echo "💡 Make sure the target ~/.local/bin is in your PATH"