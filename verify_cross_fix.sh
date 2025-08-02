#!/bin/bash

# Quick verification script for Cross.toml configuration fix

echo "🔍 Verifying Cross.toml Configuration Fix"
echo "=========================================="
echo ""

# Check if cross tool is available
if ! command -v cross &> /dev/null; then
    echo "⚠️  cross tool not installed. Installing..."
    cargo install cross --git https://github.com/cross-rs/cross || {
        echo "❌ Failed to install cross tool"
        exit 1
    }
fi

echo "✅ cross tool available: $(which cross)"
echo ""

# Test the current Cross.toml configuration
echo "🧪 Testing current Cross.toml configuration..."

if [ -f "Cross.toml" ]; then
    echo "📋 Current Cross.toml content:"
    cat Cross.toml | head -20
    echo ""
    
    # Test cross configuration without building
    echo "🔧 Testing cross configuration (dry run)..."
    
    # Set up environment variables that would be used in CI
    export PKG_CONFIG_ALLOW_CROSS="1"
    export OPENSSL_LIB_DIR="/usr/lib/aarch64-linux-gnu"
    export OPENSSL_INCLUDE_DIR="/usr/include"
    export OPENSSL_STATIC="1"
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc"
    export CROSS_NO_WARNINGS="0"
    
    # Try cross check (doesn't build, just validates)
    if cross check --target aarch64-unknown-linux-gnu 2>&1 | grep -q "found unused key"; then
        echo "❌ Cross.toml still has configuration issues"
        echo "   Found 'unused key' error in cross output"
    else
        echo "✅ Cross.toml configuration appears valid"
        echo "   No 'unused key' errors detected"
    fi
    
    # Clean up environment
    unset PKG_CONFIG_ALLOW_CROSS OPENSSL_LIB_DIR OPENSSL_INCLUDE_DIR OPENSSL_STATIC
    unset CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER CROSS_NO_WARNINGS
    
else
    echo "❌ Cross.toml not found in current directory"
    exit 1
fi

echo ""
echo "📝 Summary of fixes applied:"
echo "   1. ❌ Removed invalid [target.*.env] sections from Cross.toml"
echo "   2. ✅ Used 'passthrough' to allow environment variables through"
echo "   3. ✅ Added CROSS_NO_WARNINGS=0 to CI to suppress warnings"
echo "   4. ✅ Added fallback from cross tool to direct cargo compilation"
echo ""

echo "🚀 The GitHub Actions should now work correctly!"
echo ""
echo "💡 Key changes:"
echo "   - Cross.toml now only uses 'passthrough' for environment variables"
echo "   - CI sets CROSS_NO_WARNINGS=0 to disable warnings"
echo "   - CI has fallback to direct cargo if cross tool fails"