#!/bin/bash

# Test script to build and run the Docker cross-compilation test

set -e

echo "🐳 Testing Cross-Compilation with Docker"
echo "========================================"
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

echo "✅ Docker is available and running"
echo ""

# Build the test image
echo "🔨 Building test Docker image..."
if docker build -f Dockerfile.test-cross-compilation -t terminalai-cross-test .; then
    echo "✅ Docker image built successfully"
else
    echo "❌ Failed to build Docker image"
    exit 1
fi

echo ""
echo "🧪 Running cross-compilation test..."
echo ""

# Run the test
if docker run --rm terminalai-cross-test; then
    echo ""
    echo "✅ Cross-compilation test completed successfully!"
    echo ""
    echo "🎯 What this test verified:"
    echo "  ✅ ARM64 cross-compiler setup works"
    echo "  ✅ Vendored OpenSSL resolves architecture mismatch"
    echo "  ✅ All binaries compile successfully for ARM64"
    echo "  ✅ GitHub Actions approach should work"
else
    echo ""
    echo "❌ Cross-compilation test failed"
    echo ""
    echo "💡 This indicates the issue may need further investigation"
    exit 1
fi

echo ""
echo "🧹 Cleaning up..."
# docker rmi terminalai-cross-test 2>/dev/null || true

echo ""
echo "🎉 Docker cross-compilation test completed!"
echo ""
echo "🚀 Your GitHub Actions should now work with the vendored OpenSSL approach!"