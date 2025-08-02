#!/bin/bash

echo "🧪 Testing Terminal AI locally (simulation mode)"
echo ""

# Create a mock test environment
mkdir -p test_env
cd test_env

# Create some test files
echo "Hello World" > test1.txt
echo "Another file" > test2.txt
echo "Python code with TODO: fix this" > app.py

echo "📁 Test environment created with files:"
ls -la

echo ""
echo "🤖 Simulating cp_ai command..."
echo "Command: cp_ai \"copy all files to documents folder amir\""
echo ""
echo "Expected AI response would be:"
echo "mkdir -p ~/Documents/amir"
echo "cp * ~/Documents/amir/"
echo ""
echo "This would create the directory and copy all files."

echo ""
echo "🔍 Simulating grep_ai command..."
echo "Command: grep_ai \"find TODO comments in python files\""
echo ""
echo "Expected AI response would be:"
echo "find . -name \"*.py\" -type f -exec grep -n \"TODO\" {} +"
echo ""
echo "This would find TODO comments in Python files."

# Cleanup
cd ..
rm -rf test_env

echo ""
echo "✅ Local test simulation complete!"
echo "💡 To use on Ubuntu/Linux:"
echo "   1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "   2. Install Ollama: curl -fsSL https://ollama.ai/install.sh | sh"
echo "   3. Build: ./build.sh"
echo "   4. Install: sudo cp target/release/* /usr/local/bin/"