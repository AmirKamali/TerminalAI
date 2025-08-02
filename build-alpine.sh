#!/bin/bash

# Build script for Terminal AI Alpine Docker image

set -e

echo "🐳 Building Terminal AI Alpine Docker image..."
echo "=============================================="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    echo "   On macOS: Open Docker Desktop"
    echo "   On Linux: sudo systemctl start docker"
    exit 1
fi

# Build the Alpine image
echo "📦 Building terminalai:alpine image..."
docker build -f Dockerfile.alpine -t terminalai:alpine .

echo ""
echo "✅ Build complete!"
echo ""
echo "Image: terminalai:alpine"
echo ""
echo "You can now run:"
echo "  docker run -it --rm -v \$(pwd):/workspace terminalai:alpine"
echo "  docker-compose -f docker-compose.alpine.yml up"
echo ""