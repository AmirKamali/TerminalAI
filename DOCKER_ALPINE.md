# Terminal AI - Alpine Linux Docker Setup

This document describes how to build and use Terminal AI in a lightweight Alpine Linux Docker container.

## Features

✅ **Lightweight**: Based on Alpine Linux (~15MB base)  
✅ **All Commands Ready**: tai, cp_ai, grep_ai, find_ai  
✅ **Static Binaries**: No runtime dependencies  
✅ **Multi-stage Build**: Optimized for size  
✅ **Non-root User**: Secure by default  
✅ **Volume Support**: Easy file access  
✅ **Ollama Integration**: Optional local AI  

## Quick Start

### 1. Build the Image

```bash
# Simple build
./build-alpine.sh

# Build without cache
./build-alpine.sh --no-cache

# Build with custom tag
./build-alpine.sh --tag=terminalai:v1.0-alpine
```

### 2. Run with Docker Compose (Recommended)

```bash
# Start Terminal AI container
docker-compose -f docker-compose.alpine.yml up -d terminalai

# Access the container
docker-compose -f docker-compose.alpine.yml exec terminalai bash

# Optional: Start Ollama for local AI
docker-compose -f docker-compose.alpine.yml up -d ollama
```

### 3. Run Directly with Docker

```bash
# Interactive shell with current directory mounted
docker run -it --rm -v $(pwd):/workspace terminalai:alpine /bin/bash

# Run a specific command
docker run --rm -v $(pwd):/workspace terminalai:alpine cp_ai "copy all .txt files to backup/"

# Show help
docker run --rm terminalai:alpine terminalai-help
```

## Configuration

### AI Provider Setup

The container includes a default configuration file at `/home/terminalai/.config/terminalai/terminalai.conf`. 

#### Option 1: Mount Custom Config
```bash
docker run -it --rm \
  -v $(pwd):/workspace \
  -v $(pwd)/my-terminalai.conf:/home/terminalai/.config/terminalai/terminalai.conf \
  terminalai:alpine /bin/bash
```

#### Option 2: Edit Inside Container
```bash
docker-compose -f docker-compose.alpine.yml exec terminalai bash
vi ~/.config/terminalai/terminalai.conf
```

### Supported AI Providers

1. **Ollama (Local)**
   ```toml
   active_provider = "ollama"
   [ollama]
   url = "http://localhost:11434"
   model = "llama2"
   ```

2. **OpenAI**
   ```toml
   active_provider = "openai"
   [openai]
   api_key = "your-api-key"
   model = "gpt-3.5-turbo"
   ```

3. **Claude (Anthropic)**
   ```toml
   active_provider = "claude"
   [claude]
   api_key = "your-api-key"
   model = "claude-3-sonnet-20240229"
   ```

4. **Gemini (Google)**
   ```toml
   active_provider = "gemini"
   [gemini]
   api_key = "your-api-key"
   model = "gemini-pro"
   ```

## Complete Setup with Ollama

For a fully self-contained AI setup:

```bash
# 1. Start Ollama
docker-compose -f docker-compose.alpine.yml up -d ollama

# 2. Wait for Ollama to be ready (check logs)
docker-compose -f docker-compose.alpine.yml logs -f ollama

# 3. Pull a model
docker-compose -f docker-compose.alpine.yml exec ollama ollama pull llama2

# 4. Start Terminal AI
docker-compose -f docker-compose.alpine.yml up -d terminalai

# 5. Use Terminal AI
docker-compose -f docker-compose.alpine.yml exec terminalai bash
```

## Available Commands

🌟 **All Terminal AI commands are globally available from any directory** - just like standard Linux commands (`ls`, `cd`, `grep`, etc.)!

Once inside the container:

```bash
# Show help and available commands (works from anywhere)
terminalai-help

# Main Terminal AI interface (global command)
tai --help

# AI-powered file operations (from any directory)
cd /tmp
cp_ai "copy all .log files from last week to archive/"

cd /workspace  
cp_ai "backup my project excluding node_modules and .git"

# AI-powered search operations (from any directory)
cd /home/terminalai
grep_ai "find all TODO comments in Python files"

cd /
grep_ai "search for 'function' in JavaScript files with line numbers"

# AI-powered file finding (from any directory)
cd /var/log
find_ai "locate large files over 100MB"

cd /tmp
find_ai "find empty directories in this project"
```

💡 **No need to specify full paths** - Terminal AI commands are in your `PATH` and work from everywhere!

## Examples

### File Management
```bash
# Copy operations
cp_ai "copy all .pdf files to ~/Documents/"
cp_ai "backup config files to /backup preserving structure"

# Find operations  
find_ai "find all images larger than 5MB"
find_ai "locate Python files modified in the last 3 days"

# Search operations
grep_ai "find error messages in log files"
grep_ai "search for API keys or passwords in config files"
```

### Development Workflow
```bash
# Code analysis
grep_ai "find all TODO and FIXME comments"
find_ai "locate test files in this project"

# Log analysis
grep_ai "find errors in application logs from today"
find_ai "locate log files with recent activity"

# Cleanup
cp_ai "archive old log files to backup directory"
find_ai "find temporary or cache files to clean up"
```

## Volume Mounts

The Docker setup supports several volume mount patterns:

```bash
# Mount current directory as workspace
-v $(pwd):/workspace

# Mount specific directories
-v /home/user/projects:/projects
-v /var/logs:/logs

# Mount config file
-v $(pwd)/custom.conf:/home/terminalai/.config/terminalai/terminalai.conf
```

## Network Access

### For Ollama
- **Host Network**: Use `--network host` for direct localhost access
- **Docker Network**: Use the provided docker-compose setup
- **Remote Ollama**: Configure URL in terminalai.conf

### For External APIs
- No special network configuration needed
- Ensure API keys are properly configured

## Troubleshooting

### Build Issues
```bash
# Clear Docker cache
docker system prune -f

# Rebuild without cache
./build-alpine.sh --no-cache
```

### Runtime Issues
```bash
# Check if commands are available
docker run --rm terminalai:alpine which tai

# Test configuration
docker run --rm terminalai:alpine tai --help

# Check file permissions
docker run --rm -v $(pwd):/workspace terminalai:alpine ls -la /workspace
```

### Connection Issues
```bash
# Test Ollama connection
docker run --rm --network host terminalai:alpine curl http://localhost:11434/api/version

# Check configuration
docker run --rm terminalai:alpine cat /home/terminalai/.config/terminalai/terminalai.conf
```

## Image Details

- **Base Image**: Alpine Linux 3.19
- **Architecture**: x86_64 (amd64)
- **Size**: ~50MB (including all binaries)
- **User**: Non-root (terminalai:1000)
- **Binaries**: Statically linked
- **Config Location**: `/home/terminalai/.config/terminalai/terminalai.conf`

## Security Features

- ✅ Non-root user by default
- ✅ Minimal attack surface (Alpine Linux)
- ✅ No unnecessary packages
- ✅ Static binaries (no shared library vulnerabilities)
- ✅ Read-only recommended for production use

## Production Usage

For production environments:

```bash
# Run as read-only
docker run --read-only --rm -v $(pwd):/workspace terminalai:alpine cp_ai "your command"

# Limit resources
docker run --memory=256m --cpus=0.5 --rm terminalai:alpine tai --help

# Security options
docker run --security-opt=no-new-privileges --cap-drop=ALL --rm terminalai:alpine
```

This Alpine-based setup provides a lightweight, secure, and fully-featured Terminal AI environment that's perfect for both development and production use cases.