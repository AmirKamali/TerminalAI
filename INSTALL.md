# Installation Guide

## Prerequisites

Before installing Terminal AI, ensure you have:

1. **Rust Programming Language** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Ollama** for AI inference
   ```bash
   curl -fsSL https://ollama.ai/install.sh | sh
   ```

3. **Git** (for cloning the repository)
   ```bash
   sudo apt update && sudo apt install git
   ```

## Quick Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/username/terminalai.git
cd terminalai

# Build the project
./build.sh

# Install globally (requires sudo)
sudo cp target/release/* /usr/local/bin/

# Or install for current user only
mkdir -p ~/.local/bin
cp target/release/* ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Option 2: Using Cargo

```bash
# Clone and install directly with cargo
git clone https://github.com/username/terminalai.git
cd terminalai
cargo install --path .
```

## Setup Ollama

1. **Start Ollama service:**
   ```bash
   ollama serve
   ```

2. **Download a model (in another terminal):**
   ```bash
   ollama pull llama2
   # or
   ollama pull codellama
   # or
   ollama pull mistral
   ```

## Initialize Terminal AI

```bash
tai init
```

This will prompt you for:
- Ollama URL (default: http://localhost:11434)
- Model name (default: llama2)
- Timeout in seconds (default: 30)

## Verify Installation

Test the installation with:

```bash
# Check main command
tai

# Test AI commands
cp_ai "copy all .txt files to backup folder"
grep_ai "find all functions in python files"
```

## Troubleshooting

### "command not found"

- Ensure the binaries are in your PATH
- For system installation: check `/usr/local/bin` is in PATH
- For user installation: ensure `~/.local/bin` is in PATH

### "Ollama connection failed"

- Start Ollama: `ollama serve`
- Check if model is available: `ollama list`
- Reconfigure: `tai init`

### Build errors

- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build --release`