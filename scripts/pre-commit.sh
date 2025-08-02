#!/bin/bash

# Terminal AI - Pre-commit Hook
# This script runs quality checks before each commit

set -e

echo "🔍 Running pre-commit quality checks..."
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✅ $message${NC}"
    elif [ "$status" = "FAIL" ]; then
        echo -e "${RED}❌ $message${NC}"
    else
        echo -e "${YELLOW}⚠️  $message${NC}"
    fi
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not in a git repository. Skipping pre-commit checks."
    exit 0
fi

# Get the list of staged Rust files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo "ℹ️  No Rust files staged. Skipping Rust-specific checks."
else
    echo "📋 Staged Rust files:"
    echo "$STAGED_FILES" | sed 's/^/  - /'
    echo ""
fi

# 1. Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_status "FAIL" "Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# 2. Run cargo fmt check
echo "🔧 Checking code formatting..."
if cargo fmt --all -- --check; then
    print_status "PASS" "Code formatting check passed"
else
    print_status "FAIL" "Code formatting check failed. Run 'cargo fmt --all' to fix."
    echo "💡 To fix formatting issues, run: cargo fmt --all"
    exit 1
fi

# 3. Run cargo check
echo "🧹 Running cargo check..."
if cargo check --all-targets --all-features; then
    print_status "PASS" "Cargo check passed"
else
    print_status "FAIL" "Cargo check failed"
    exit 1
fi

# 4. Run clippy with strict settings
echo "🔍 Running clippy linting..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_status "PASS" "Clippy linting passed"
else
    print_status "FAIL" "Clippy linting failed"
    echo "💡 Common issues to fix:"
    echo "  - Use inline format strings: format!(\"{variable}\") instead of format!(\"{}\", variable)"
    echo "  - Remove unused imports"
    echo "  - Use descriptive error messages"
    exit 1
fi

# 5. Run tests (only if there are staged Rust files)
if [ -n "$STAGED_FILES" ]; then
    echo "🧪 Running tests..."
    if cargo test; then
        print_status "PASS" "All tests passed"
    else
        print_status "FAIL" "Tests failed"
        exit 1
    fi
else
    print_status "SKIP" "Skipping tests (no Rust files staged)"
fi

# 6. Check for common issues in staged files
if [ -n "$STAGED_FILES" ]; then
    echo "🔍 Checking for common issues in staged files..."
    
    # Check for old format string patterns
    OLD_FORMAT_PATTERNS=$(git diff --cached | grep -E 'format!\(".*\{.*\}.*",' || true)
    if [ -n "$OLD_FORMAT_PATTERNS" ]; then
        print_status "FAIL" "Found old format string patterns. Use inline format strings."
        echo "💡 Replace format!(\"{}\", variable) with format!(\"{variable}\")"
        echo "💡 Replace println!(\"{}\", variable) with println!(\"{variable}\")"
        exit 1
    fi
    
    # Check for unused imports
    UNUSED_IMPORTS=$(git diff --cached | grep -E 'use .*;.*// unused' || true)
    if [ -n "$UNUSED_IMPORTS" ]; then
        print_status "FAIL" "Found unused imports. Remove them before committing."
        exit 1
    fi
    
    print_status "PASS" "No common issues found in staged files"
fi

# 7. Optional: Run cross-compilation test (only if explicitly requested)
if [ "$1" = "--with-cross-compilation" ]; then
    echo "🐳 Running cross-compilation test..."
    if [ -f "./test_docker_cross_compilation.sh" ]; then
        if ./test_docker_cross_compilation.sh; then
            print_status "PASS" "Cross-compilation test passed"
        else
            print_status "FAIL" "Cross-compilation test failed"
            exit 1
        fi
    else
        print_status "SKIP" "Cross-compilation test script not found"
    fi
fi

echo ""
echo "🎉 All pre-commit checks passed!"
echo ""
echo "📋 Summary:"
echo "  ✅ Code formatting: OK"
echo "  ✅ Cargo check: OK"
echo "  ✅ Clippy linting: OK"
echo "  ✅ Tests: OK"
echo "  ✅ Common issues: OK"
echo ""
echo "💡 Quality gates passed - ready to commit!"
echo ""
echo "🔧 For future reference:"
echo "  - Run 'cargo fmt --all' to format code"
echo "  - Run 'cargo clippy --all-targets --all-features -- -D warnings' to check linting"
echo "  - Run 'cargo test' to run tests"
echo "  - Run './test_docker_cross_compilation.sh' to test cross-compilation"

exit 0 