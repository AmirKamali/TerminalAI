#!/bin/bash

echo "🧪 Testing Terminal AI Binary Independence"
echo "==========================================="

# Test that binaries work independently without source files

echo ""
echo "📁 Creating isolated test environment..."
mkdir -p /tmp/terminalai_test
cd /tmp/terminalai_test

echo "📋 Current directory: $(pwd)"
echo "📋 Directory contents: $(ls -la)"

echo ""
echo "🔧 Testing if we have the binaries (should be in PATH after installation)..."

# Check if binaries exist
if ! command -v tai &> /dev/null; then
    echo "❌ tai not found in PATH"
    echo "💡 Make sure to run: sudo cp target/release/* /usr/local/bin/"
    exit 1
fi

if ! command -v cp_ai &> /dev/null; then
    echo "❌ cp_ai not found in PATH"
    exit 1
fi

if ! command -v grep_ai &> /dev/null; then
    echo "❌ grep_ai not found in PATH"
    exit 1
fi

echo "✅ All binaries found in PATH"

echo ""
echo "📝 Creating test files..."
echo "Test content 1" > file1.txt
echo "Test content 2" > file2.txt
echo "Python code with TODO: fix this" > test.py

echo ""
echo "🧪 Test 1: Valid cp_ai operation (should work)"
echo "Command: cp_ai \"copy all txt files to backup folder\""
echo "Expected: Should work and copy files"

echo ""
echo "🧪 Test 2: Invalid cp_ai operation (should reject)"
echo "Command: cp_ai \"find all TODO comments\""
echo "Expected: Should reject and suggest tai -p"

echo ""
echo "🧪 Test 3: Valid grep_ai operation (should work)"
echo "Command: grep_ai \"find TODO in python files\""
echo "Expected: Should work and search files"

echo ""
echo "🧪 Test 4: Invalid grep_ai operation (should reject)"
echo "Command: grep_ai \"copy files to backup\""
echo "Expected: Should reject and suggest tai -p"

echo ""
echo "🧪 Test 5: Complex orchestration"
echo "Command: tai -p \"backup txt files and find TODO comments\""
echo "Expected: Should break down into cp_ai and grep_ai commands"

echo ""
echo "✅ Binary independence test setup complete!"
echo "💡 Run the above commands manually to verify independence"
echo "💡 All binaries should work without access to source cmd/ files"

# Cleanup
echo ""
echo "🧹 Cleaning up test environment..."
cd /
rm -rf /tmp/terminalai_test

echo "🎉 Test environment cleanup complete!"