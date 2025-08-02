#!/bin/bash

# Ubuntu CI Build Script for Terminal AI
# This script is specifically designed for GitHub Actions and Ubuntu environments

set -e

echo "🚀 Building Terminal AI on Ubuntu CI..."
echo "📋 Environment: Ubuntu with cross-compilation support"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Install system dependencies for cross-compilation
echo "📦 Installing system dependencies..."
sudo apt-get update -qq
sudo apt-get install -y build-essential pkg-config

# Install ARM64 cross-compilation tools
echo "🔧 Setting up ARM64 cross-compilation..."
sudo apt-get install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    crossbuild-essential-arm64 \
    libc6-dev-arm64-cross

# Install OpenSSL (host version for static linking)
echo "🔐 Installing OpenSSL development files..."
sudo apt-get install -y libssl-dev

echo "✅ System dependencies installed"

# Add Rust targets
echo "🔧 Adding Rust targets..."
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu

# Optional: Install cross tool for better cross-compilation support
echo "🛠️  Installing cross tool for enhanced cross-compilation..."
if command -v cross &> /dev/null; then
    echo "  ✅ cross tool already installed"
else
    cargo install cross --git https://github.com/cross-rs/cross || echo "  ⚠️  Failed to install cross tool, falling back to cargo"
fi

# Create output directories
echo "📁 Creating output directories..."
mkdir -p release/linux-arm64
mkdir -p release/linux-x86_64

# Function to build for a specific target
build_target() {
    local target=$1
    local platform_name=$2
    local output_dir=$3
    
    echo "🔨 Building for $platform_name ($target)..."
    
    # Set up environment for cross-compilation
    if [[ "$target" == "aarch64-unknown-linux-gnu" ]]; then
        echo "🔧 Setting up cross-compilation environment for ARM64..."
        
        # Set cross-compilation environment variables (using vendored OpenSSL)
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc"
        export PKG_CONFIG_ALLOW_CROSS=1
        export CC_aarch64_unknown_linux_gnu="aarch64-linux-gnu-gcc"
        export CXX_aarch64_unknown_linux_gnu="aarch64-linux-gnu-g++"
        
        # Debug output
        echo "  📋 Cross-compilation setup:"
        echo "    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER"
        echo "    PKG_CONFIG_ALLOW_CROSS=$PKG_CONFIG_ALLOW_CROSS"
        echo "    CC=$CC_aarch64_unknown_linux_gnu"
        echo "    CXX=$CXX_aarch64_unknown_linux_gnu"
        echo "    Using vendored OpenSSL to avoid architecture mismatch"
        
        # Verify ARM64 GCC setup
        which aarch64-linux-gnu-gcc >/dev/null 2>&1 || echo "    ⚠️  ARM64 GCC not found in PATH"
        aarch64-linux-gnu-gcc --version >/dev/null 2>&1 || echo "    ⚠️  ARM64 GCC not working"
    fi
    
    # Build for target - try cross first for ARM64, fallback to cargo
    local build_success=false
    
    if [[ "$target" == "aarch64-unknown-linux-gnu" ]] && command -v cross &> /dev/null; then
        echo "  🔧 Using cross tool for ARM64 compilation..."
        if cross build --release --target $target; then
            build_success=true
            echo "  ✅ cross build successful"
        else
            echo "  ⚠️  cross build failed, falling back to cargo..."
        fi
    fi
    
    # Fallback to cargo if cross didn't work or wasn't used
    if [ "$build_success" = false ]; then
        echo "  🔧 Using cargo for compilation..."
        if [[ "$target" == "aarch64-unknown-linux-gnu" ]]; then
            echo "    Building ARM64 with vendored OpenSSL..."
            if cargo build --release --target $target --features vendored-openssl; then
                build_success=true
            fi
        else
            echo "    Building natively..."
            if cargo build --release --target $target; then
                build_success=true
            fi
        fi
    fi
    
    if [ "$build_success" = true ]; then
        echo "✅ Build successful for $platform_name"
        
        # Copy binaries to appropriate directory
        local target_dir="target/$target/release"
        
            if [ -f "$target_dir/tai" ]; then
        cp "$target_dir/tai" "$output_dir/"
            cp "$target_dir/cp_ai" "$output_dir/"
            cp "$target_dir/grep_ai" "$output_dir/"
            # Copy find_ai if it exists
            if [ -f "$target_dir/find_ai" ]; then
                cp "$target_dir/find_ai" "$output_dir/"
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
        echo "❌ Build failed for $platform_name"
        exit 1
    fi
}

# Build for Linux targets
echo "🎯 Building for Linux targets..."

# Build x86_64
build_target "x86_64-unknown-linux-gnu" "Linux x86_64" "release/linux-x86_64"

# Build ARM64
build_target "aarch64-unknown-linux-gnu" "Linux ARM64" "release/linux-arm64"

echo "✅ All builds completed successfully!"
echo ""
echo "📋 Build Summary:"
echo "  - Linux x86_64: release/linux-x86_64/"
echo "  - Linux ARM64: release/linux-arm64/"
echo ""
echo "📦 Binaries created:"
for dir in release/linux-*; do
    if [ -d "$dir" ]; then
        echo "  $dir:"
        ls -la "$dir" | grep -E "(tai|cp_ai|grep_ai|find_ai|terminalai.conf)" || echo "    (no files found)"
    fi
done 