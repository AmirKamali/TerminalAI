#!/bin/bash

# Quick start script for Terminal AI Alpine Docker setup

set -e

echo "🐳 Terminal AI - Alpine Docker Quick Start"
echo "=========================================="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    echo "   On macOS: Open Docker Desktop"
    echo "   On Linux: sudo systemctl start docker"
    exit 1
fi

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose not found. Please install docker-compose."
    exit 1
fi

echo "✅ Docker is running"
echo ""

# Menu for user choice
echo "Choose an option:"
echo "1. Build Alpine image only"
echo "2. Build and run Terminal AI container"
echo "3. Build and run with Ollama (complete AI setup)"
echo "4. Just run existing image"
echo "5. Show help and examples"
echo ""
read -p "Enter your choice (1-5): " choice

case $choice in
    1)
        echo "🔨 Building Alpine image..."
        ./build-alpine.sh
        echo ""
        echo "✅ Build complete! You can now run:"
        echo "   docker run -it --rm -v \$(pwd):/workspace terminalai:alpine /bin/bash"
        ;;
    2)
        echo "🔨 Building and starting Terminal AI..."
        ./build-alpine.sh
        docker-compose -f docker-compose.alpine.yml up -d terminalai
        echo ""
        echo "✅ Terminal AI is running! Access it with:"
        echo "   docker-compose -f docker-compose.alpine.yml exec terminalai bash"
        echo ""
        echo "To stop: docker-compose -f docker-compose.alpine.yml down"
        ;;
    3)
        echo "🔨 Building and starting complete AI setup..."
        ./build-alpine.sh
        
        echo "🚀 Starting Ollama..."
        docker-compose -f docker-compose.alpine.yml up -d ollama
        
        echo "⏳ Waiting for Ollama to be ready..."
        sleep 10
        
        echo "📦 Pulling Llama2 model (this may take a while)..."
        docker-compose -f docker-compose.alpine.yml exec ollama ollama pull llama2
        
        echo "🚀 Starting Terminal AI..."
        docker-compose -f docker-compose.alpine.yml up -d terminalai
        
        echo ""
        echo "✅ Complete setup ready! Access Terminal AI with:"
        echo "   docker-compose -f docker-compose.alpine.yml exec terminalai bash"
        echo ""
        echo "Inside the container, try:"
        echo "   tai --help"
        echo "   cp_ai 'copy all .txt files to backup/'"
        echo ""
        echo "To stop everything: docker-compose -f docker-compose.alpine.yml down"
        ;;
    4)
        echo "🚀 Starting existing image..."
        if docker images | grep -q "terminalai.*alpine"; then
            docker-compose -f docker-compose.alpine.yml up -d terminalai
            echo ""
            echo "✅ Terminal AI is running! Access it with:"
            echo "   docker-compose -f docker-compose.alpine.yml exec terminalai bash"
        else
            echo "❌ No Terminal AI Alpine image found. Please build first with option 1 or 2."
        fi
        ;;
    5)
        echo "📖 Help and Examples"
        echo "==================="
        echo ""
        echo "Basic Usage:"
        echo "  # Access Terminal AI container"
        echo "  docker-compose -f docker-compose.alpine.yml exec terminalai bash"
        echo ""
        echo "  # Run single commands"
        echo "  docker run --rm -v \$(pwd):/workspace terminalai:alpine cp_ai 'copy logs to backup'"
        echo ""
        echo "Available Commands:"
        echo "  tai  - Main interface"
        echo "  cp_ai        - AI-powered copying"
        echo "  grep_ai      - AI-powered searching"
        echo "  find_ai      - AI-powered file finding"
        echo ""
        echo "Examples:"
        echo "  cp_ai 'backup all .pdf files to Documents/'"
        echo "  grep_ai 'find TODO comments in Python files'"
        echo "  find_ai 'locate files larger than 100MB'"
        echo ""
        echo "Configuration:"
        echo "  Edit: ~/.config/terminalai/terminalai.conf"
        echo "  Providers: ollama, openai, claude, gemini"
        echo ""
        echo "For detailed documentation, see: DOCKER_ALPINE.md"
        ;;
    *)
        echo "❌ Invalid choice. Please run the script again and choose 1-5."
        exit 1
        ;;
esac

echo ""
echo "🎉 Done! Enjoy using Terminal AI on Alpine Linux!"