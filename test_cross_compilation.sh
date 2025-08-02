#!/bin/bash

# Test script to verify cross-compilation fixes
# This script helps test the OpenSSL cross-compilation setup locally
# and replicates the exact GitHub Actions environment

set -e

echo "🧪 Testing Cross-Compilation Setup"
echo "=================================="
echo ""

# Test function to replicate GitHub Actions cross tool issue
test_cross_tool_issue() {
    echo "🔍 Testing cross tool configuration issue replication..."
    
    # Check if cross tool is available
    if ! command -v cross &> /dev/null; then
        echo "  ⚠️  cross tool not installed, installing..."
        cargo install cross --git https://github.com/cross-rs/cross || {
            echo "  ❌ Failed to install cross tool"
            return 1
        }
    fi
    
    # Create a problematic Cross.toml to replicate the exact error
    local temp_cross_toml="Cross.toml.test"
    cat > "$temp_cross_toml" << 'EOF'
[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"

[target.aarch64-unknown-linux-gnu.env]
OPENSSL_STATIC = "1"
PKG_CONFIG_ALLOW_CROSS = "1"
EOF

    echo "  🚨 Testing with problematic Cross.toml configuration..."
    echo "     This should cause: 'found unused key(s) in Cross configuration'"
    
    # Try to run cross with the problematic config
    if CROSS_CONFIG="$temp_cross_toml" cross build --release --target aarch64-unknown-linux-gnu --bin tai 2>&1 | grep -q "found unused key"; then
        echo "  ✅ Successfully reproduced the cross tool configuration error!"
        echo "     Error: found unused key(s) in Cross configuration"
    else
        echo "  ⚠️  Could not reproduce the exact cross tool error"
    fi
    
    # Clean up
    rm -f "$temp_cross_toml"
    echo "  🧹 Cleaned up test configuration"
}

# Test function to replicate original OpenSSL environment issue  
test_openssl_env_issue() {
    echo "🔍 Testing original OpenSSL environment issue replication..."
    
    # This replicates the problematic conditional environment variables
    # that caused the issue in GitHub Actions
    local target="x86_64-unknown-linux-gnu"  # Non-ARM64 target
    
    # Simulate the problematic conditional logic from GitHub Actions
    export OPENSSL_LIB_DIR=""  # This empty string causes the panic!
    export OPENSSL_INCLUDE_DIR=""
    export PKG_CONFIG_PATH=""
    
    echo "  🚨 Reproducing the empty string error condition:"
    echo "     OPENSSL_LIB_DIR='$OPENSSL_LIB_DIR'"
    echo "     OPENSSL_INCLUDE_DIR='$OPENSSL_INCLUDE_DIR'"
    echo "     PKG_CONFIG_PATH='$PKG_CONFIG_PATH'"
    
    echo "  🔨 Attempting build with empty OPENSSL_LIB_DIR (this should fail)..."
    
    # This should reproduce the exact error we saw
    if timeout 30 cargo build --release --target "$target" --bin tai 2>&1 | grep -q "OpenSSL library directory does not exist"; then
        echo "  ✅ Successfully reproduced the GitHub Actions error!"
        echo "     Error: OpenSSL library directory does not exist with empty string"
    else
        echo "  ⚠️  Could not reproduce the exact error (different environment?)"
    fi
    
    # Clean up the problematic environment variables
    unset OPENSSL_LIB_DIR
    unset OPENSSL_INCLUDE_DIR 
    unset PKG_CONFIG_PATH
    
    echo "  🧹 Cleaned up problematic environment variables"
}

# Check if we're on a supported platform
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ This test script is designed for Linux environments"
    echo "   For other platforms, use the GitHub Actions workflows"
    exit 1
fi

# Function to test a specific target
test_target() {
    local target=$1
    local friendly_name=$2
    
    echo "🎯 Testing $friendly_name ($target)..."
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "$target"; then
        echo "  📦 Installing target $target..."
        rustup target add "$target"
    fi
    
    # Try to build a simple test
    echo "  🔨 Building tai for $target..."
    
    if [[ "$target" == "aarch64-unknown-linux-gnu" ]]; then
        # Test ARM64 cross-compilation with proper environment
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc"
        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/pkgconfig"
        export OPENSSL_LIB_DIR="/usr/lib/aarch64-linux-gnu"
        export OPENSSL_INCLUDE_DIR="/usr/include"
        export OPENSSL_STATIC=1
        export CC_aarch64_unknown_linux_gnu="aarch64-linux-gnu-gcc"
        export CXX_aarch64_unknown_linux_gnu="aarch64-linux-gnu-g++"
        
        # Check dependencies
        if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
            echo "  ❌ aarch64-linux-gnu-gcc not found"
            echo "     Install with: sudo apt-get install gcc-aarch64-linux-gnu"
            return 1
        fi
        
        if [ ! -f "/usr/lib/aarch64-linux-gnu/libssl.so" ]; then
            echo "  ❌ ARM64 OpenSSL not found"
            echo "     Install with: sudo apt-get install libssl-dev:arm64"
            return 1
        fi
    fi
    
    # Try cross tool first, then fallback to cargo
    local build_success=false
    
    if [[ "$target" == "aarch64-unknown-linux-gnu" ]] && command -v cross &> /dev/null; then
        echo "  🔧 Trying with cross tool..."
        if cross build --release --target "$target" --bin tai; then
            build_success=true
            echo "  ✅ cross build successful!"
        else
            echo "  ⚠️  cross build failed, trying cargo..."
        fi
    fi
    
    if [ "$build_success" = false ]; then
        echo "  🔧 Trying with cargo..."
        if [[ "$target" == "aarch64-unknown-linux-gnu" ]]; then
            echo "    Building ARM64 with vendored OpenSSL..."
            if cargo build --release --target "$target" --bin tai --features vendored-openssl; then
                build_success=true
                echo "  ✅ cargo build successful with vendored OpenSSL!"
            fi
        else
            echo "    Building natively..."
            if cargo build --release --target "$target" --bin tai; then
                build_success=true
                echo "  ✅ cargo build successful!"
            fi
        fi
    fi
    
    if [ "$build_success" = true ]; then
        # Verify binary was created
        local binary_path="target/$target/release/tai"
        if [ -f "$binary_path" ]; then
            local binary_size=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null || echo "unknown")
            echo "  📦 Binary created: $binary_path (${binary_size} bytes)"
            
            # Check binary info
            if command -v file &> /dev/null; then
                echo "  🔍 Binary info: $(file "$binary_path")"
            fi
            
            return 0
        else
            echo "  ❌ Binary not found at $binary_path"
            return 1
        fi
    else
        echo "  ❌ Build failed for $target"
        return 1
    fi
}

# Test function to replicate OpenSSL header architecture mismatch issue
test_openssl_header_mismatch() {
    echo "🔍 Testing OpenSSL header architecture mismatch issue..."
    
    # Check if we're on a system where we can test this
    if ! command -v apt-get &> /dev/null; then
        echo "  ⚠️  apt-get not found, skipping OpenSSL header test"
        return 0
    fi
    
    echo "  🚨 Testing OpenSSL header mismatch that causes build failures..."
    echo "     This replicates: 'fatal error: openssl/opensslconf.h: No such file or directory'"
    
    # Check if ARM64 cross-compiler is available
    if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
        echo "  📦 Installing ARM64 cross-compiler for test..."
        sudo apt-get update -qq
        sudo apt-get install -y gcc-aarch64-linux-gnu 2>/dev/null || {
            echo "  ❌ Failed to install ARM64 cross-compiler"
            return 1
        }
    fi
    
    # Test the exact scenario that fails in GitHub Actions
    echo "  🔧 Testing ARM64 compiler with x86_64 OpenSSL headers..."
    
    # Create a simple test to replicate the openssl-sys issue
    local test_file="/tmp/openssl_test_$$.c"
    cat > "$test_file" << 'EOF'
#include <openssl/opensslv.h>
#include <openssl/opensslconf.h>
int main() { return 0; }
EOF
    
    # Try to compile with ARM64 compiler against host OpenSSL headers
    echo "  🔨 Testing: aarch64-linux-gnu-gcc with /usr/include/openssl..."
    if aarch64-linux-gnu-gcc -I/usr/include -o /tmp/test_openssl_$$ "$test_file" 2>&1 | grep -q "fatal error.*opensslconf.h"; then
        echo "  ✅ Successfully reproduced the OpenSSL header mismatch error!"
        echo "     Error: ARM64 compiler cannot process x86_64 OpenSSL headers"
    else
        echo "  ℹ️  Test completed without expected error"
        echo "     This might mean OpenSSL headers are compatible or missing"
    fi
    
    # Clean up test files
    rm -f "$test_file" /tmp/test_openssl_$$
    
    echo "  💡 The fix: Use vendored OpenSSL feature: cargo build --features vendored-openssl"
}

# Test function to replicate APT repository ARM64 issue  
test_apt_arm64_issue() {
    echo "🔍 Testing APT ARM64 repository issue replication..."
    
    # Check if we're on Ubuntu/Debian (where this issue occurs)
    if ! command -v apt-get &> /dev/null; then
        echo "  ⚠️  apt-get not found, skipping APT-specific test"
        return 0
    fi
    
    echo "  🚨 Testing the ARM64 repository issue that causes GitHub Actions to fail..."
    echo "     This replicates: 'Failed to fetch ...binary-arm64/Packages 404 Not Found'"
    
    # Save current sources.list for restoration
    local backup_sources="/tmp/sources.list.backup.$$"
    sudo cp /etc/apt/sources.list "$backup_sources" 2>/dev/null || echo "No sources.list to backup"
    
    # Test adding ARM64 architecture (this is what caused the 404 errors)
    echo "  🔧 Adding ARM64 architecture (this is what triggers the issue)..."
    if sudo dpkg --add-architecture arm64 2>/dev/null; then
        echo "     ARM64 architecture added"
        
        # Try apt update - this should show 404 errors for ARM64 packages
        echo "  📦 Running apt update to see if ARM64 repositories are accessible..."
        if sudo apt-get update 2>&1 | grep -q "Failed to fetch.*binary-arm64.*404"; then
            echo "  ✅ Successfully reproduced the GitHub Actions APT error!"
            echo "     Error: Failed to fetch ARM64 packages (404 Not Found)"
        else
            echo "  ℹ️  APT update succeeded or different error occurred"
            echo "     This might mean the repositories are accessible in this environment"
        fi
        
        # Clean up - remove ARM64 architecture
        echo "  🧹 Removing ARM64 architecture to clean up..."
        sudo dpkg --remove-architecture arm64 2>/dev/null || echo "Failed to remove ARM64 architecture"
        
        # Restore sources.list if we backed it up
        if [ -f "$backup_sources" ]; then
            sudo cp "$backup_sources" /etc/apt/sources.list 2>/dev/null || echo "Failed to restore sources.list"
            rm -f "$backup_sources"
        fi
        
        # Update apt cache to clean state
        sudo apt-get update -qq >/dev/null 2>&1 || echo "APT update cleanup failed"
        
    else
        echo "  ❌ Failed to add ARM64 architecture"
    fi
    
    echo "  💡 The fix: Don't add ARM64 architecture globally - use vendored OpenSSL"
}

# Test GitHub Actions issues
echo "🧪 Testing GitHub Actions Issues"
echo "--------------------------------"

# Test the OpenSSL header mismatch issue (current problem)
test_openssl_header_mismatch

echo ""

# Test the APT ARM64 repository issue (previous problem)
test_apt_arm64_issue

echo ""

# Test the cross tool configuration issue (previous problem)
test_cross_tool_issue

echo ""

# Test the original OpenSSL environment issue (fixed)  
test_openssl_env_issue

echo ""
echo "📋 Testing native compilation..."
test_target "x86_64-unknown-linux-gnu" "Linux x86_64"

echo ""
echo "📋 Testing cross-compilation..."
test_target "aarch64-unknown-linux-gnu" "Linux ARM64"

echo ""
echo "✅ Cross-compilation test completed!"
echo ""
echo "💡 If ARM64 tests failed, make sure you have:"
echo "   - sudo apt-get install gcc-aarch64-linux-gnu"
echo "   - sudo apt-get install libssl-dev:arm64"
echo "   - cross tool: cargo install cross"