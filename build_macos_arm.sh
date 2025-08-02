#!/bin/bash

# Build script for Terminal AI - Apple Silicon (ARM64) only
# This is a convenience script that calls the main build.sh with ARM64 macOS arguments

set -e

echo "🍎 Building Terminal AI for Apple Silicon (ARM64)..."

# Check if we're on macOS
CURRENT_OS=$(uname -s)
if [[ "$CURRENT_OS" != "Darwin" ]]; then
    echo "❌ This script requires macOS to build for Apple Silicon"
    exit 1
fi

# Check if build.sh exists
if [ ! -f "./build.sh" ]; then
    echo "❌ build.sh not found in current directory"
    exit 1
fi

# Call the main build script with ARM64 macOS arguments
echo "🔄 Calling main build script for ARM64 macOS..."
./build.sh --arch=arm64 --platform=macos

echo ""
echo "✅ Apple Silicon build complete via build.sh!"
echo "📁 Binaries are located in release/macos-arm64/"