# Terminal AI 🤖

AI-powered Linux commands that supercharge your terminal experience with natural language processing.

## Overview

Terminal AI provides intelligent versions of common Linux commands that understand natural language prompts and automatically execute the appropriate commands. Currently supports:

- **cp_ai** - AI-powered file copying operations
- **grep_ai** - AI-powered text search and pattern matching
- **find_ai** - AI-powered file and directory search operations
- **ps_ai** - AI-powered process related operations

## ✨ Multi-Provider Support

Choose from multiple AI providers:
- 🦙 **Ollama** (Local) - Privacy-focused, runs offline
- 🤖 **OpenAI** - GPT-3.5/GPT-4 models
- 🧠 **Claude** (Anthropic) - Advanced reasoning capabilities
- 💎 **Gemini** (Google) - Google's latest AI technology

- **AI Provider** - Choose one:
  - **LLamaCPP** (local) - Use hugging face models
  - **Ollama** (local) - [Install Ollama](https://ollama.ai/) for privacy and offline usage
  - **OpenAI API** - Get your API key from [OpenAI Platform](https://platform.openai.com/)
  - **Anthropic API** - Get your API key from [Anthropic Console](https://console.anthropic.com/)
  - **Google AI API** - Get your API key from [Google AI Studio](https://makersuite.google.com/app/apikey)
- **Linux/macOS/Windows** - Cross-platform support

## Installation

### Building from Source

1. **Clone the repository:**
   ```bash
   git clone https://github.com/username/terminalai.git
   cd terminalai
   ```

2. **Build the binaries:**
   ```bash
   cargo build --release
   ```

3. **Install binaries to your PATH:**
   ```bash
   export PATH="[PATH_TO_BINARIES_FOLDER]:$PATH" 
   e.g.: export PATH="/Users/alex/Documents/terminalai-macos-x86_64:$PATH"
   ```

### Docker Alpine (Lightweight Container)

For a lightweight, containerized setup with all commands ready to use:

1. **Quick start:**
   ```bash
   ./docker-quick-start.sh
   ```

2. **Manual build and run:**
   ```bash
   # Build Alpine image
   ./build-alpine.sh
   
   # Run with Docker Compose (recommended)
   docker-compose -f docker-compose.alpine.yml up -d
   docker-compose -f docker-compose.alpine.yml exec terminalai bash
   
   # Or run directly
   docker run -it --rm -v $(pwd):/workspace terminalai:alpine /bin/bash
   ```

3. **With local Ollama AI:**
   ```bash
   # Start everything (Terminal AI + Ollama)
   docker-compose -f docker-compose.alpine.yml up -d
   
   # Pull AI model
   docker-compose -f docker-compose.alpine.yml exec ollama ollama pull llama2

Docker benefits:
- ✅ Optional integrated Ollama for local AI
- ✅ Non-root user for security
- ✅ Volume mounts for easy file access

📖 **For detailed Docker documentation, see [DOCKER_ALPINE.md](DOCKER_ALPINE.md)**

## Setup

**Initialize Terminal AI:**
```bash
tai init
```

This will prompt you to:
1. **Select your AI provider** (Ollama, LLamaCPP, OpenAI, Claude, or Gemini)
2. **Configure provider-specific settings**:
   - **Ollama**: URL and model name  
   - **OpenAI**: API key and model selection
   - **Claude**: API key and model selection
   - **Gemini**: API key and model selection
3. **Set request timeout** (default: 30 seconds)

### Provider-Specific Setup

#### For Ollama (Local):
```bash 
# Start Ollama service
ollama serve

# Pull a model 
ollama pull llama2
# or for coding tasks:
ollama pull codellama
```

#### For Cloud Providers:
- **OpenAI**: Get your API key from [OpenAI Platform](https://platform.openai.com/)
- **Claude**: Get your API key from [Anthropic Console](https://console.anthropic.com/)  
- **Gemini**: Get your API key from [Google AI Studio](https://makersuite.google.com/app/apikey)

📚 **Detailed Setup Guide**: See [MULTI_PROVIDER_GUIDE.md](MULTI_PROVIDER_GUIDE.md) for comprehensive configuration instructions.

## Usage

### cp_ai - AI-Powered File Operations

```bash
cp_ai "copy all .txt files from current directory to ~/documents"
# → mkdir -p ~/documents && cp *.txt ~/documents/

cp_ai "backup my config files to /backup preserving permissions"
# → mkdir -p /backup && cp -p ~/.config/* /backup/

cp_ai "copy entire project folder to /tmp but exclude node_modules"
# → mkdir -p /tmp && rsync -av --exclude=node_modules . /tmp/

cp_ai "duplicate file.txt as file_backup.txt"
# → cp file.txt file_backup.txt
```

### grep_ai - AI-Powered Text Search

```bash
grep_ai "find all TODO comments in Python files"
# → find . -name "*.py" -type f -exec grep -n "TODO" {} +

grep_ai "search for email addresses in log files recursively"
# → find . -name "*.log" -type f -exec grep -E "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}" {} +

grep_ai "find functions named 'connect' in .js files ignoring case"
# → find . -name "*.js" -type f -exec grep -in "function.*connect" {} +

grep_ai "show lines containing 'error' with 3 lines of context"
# → grep -r -C 3 "error" .
```

### find_ai - AI-Powered File and Directory Search

```bash
find_ai "find all PDF files in documents folder"
# → find ~/documents -name "*.pdf" -type f

find_ai "locate large video files bigger than 1GB"
# → find . -size +1G -type f -name "*.mp4" -o -name "*.avi" -o -name "*.mov"

find_ai "find files modified in the last week"
# → find . -type f -mtime -7

find_ai "search for executable shell scripts"
# → find . -name "*.sh" -type f -executable

find_ai "find empty directories"
# → find . -type d -empty

find_ai "locate config files in home directory"
# → find ~ -name "*config*" -type f -maxdepth 3

find_ai "find Python files larger than 10MB modified recently"
# → find . -name "*.py" -type f -size +10M -mtime -7
```

**Safety Features:**
- All generated commands are shown to the user before execution
- User confirmation required before running any command
- Clear error messages and command explanations

## Configuration

Configuration is stored in `~/.config/terminalai/config.json`:

```json
{
  "ollama_url": "http://localhost:11434",
  "model_name": "llama2",
  "timeout_seconds": 30
}
```

You can edit this file directly or run `tai init` to reconfigure.


## Troubleshooting

### "Connection refused" or Ollama errors

- Ensure Ollama is running: `ollama serve`
- Check if the model is available: `ollama list`
- Verify your configuration: `cat ~/.config/terminalai/config.json`

### "No executable commands found in AI response"

- The AI response didn't contain recognizable commands
- Try rephrasing your request more specifically
- Check that Ollama is responding with appropriate command suggestions

### Permission denied

- Make sure the binaries are executable: `chmod +x tai cp_ai grep_ai`
- Check that the installation directory is in your PATH

## Contributing
To add additional capablity please follow instructions [PULL_REQUEST_CHECKLIST.md](PULL_REQUEST_CHECKLIST.md)


## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Ollama](https://ollama.ai/) for providing local AI inference
- The Rust community for excellent tooling and libraries
- All contributors who help make terminal experiences better

---

**Note:** This project is in early development. Please report issues and provide feedback to help improve the tool.