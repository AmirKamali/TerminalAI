#!/bin/bash

# Format Project Script for Terminal AI
# This script runs formatting and linting for the Rust project

set -e

# Parse command line arguments
AUTO_FIX=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --fix)
            AUTO_FIX=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --fix     Auto-fix formatting issues instead of just checking"
            echo "  --help    Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0              # Check formatting and linting (CI mode)"
            echo "  $0 --fix        # Auto-fix formatting issues"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

if [ "$AUTO_FIX" = true ]; then
    echo "🎨 Formatting Terminal AI project (Auto-fix mode)..."
    echo "📋 This script will auto-fix formatting issues and run all checks"
else
    echo "🎨 Formatting Terminal AI project (GitHub CI mode)..."
    echo "📋 This script replicates GitHub CI checks for local development"
fi
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Check if rustfmt is installed
if ! command -v rustfmt &> /dev/null; then
    echo "📦 Installing rustfmt..."
    rustup component add rustfmt
fi

# Check if clippy is installed
if ! command -v cargo-clippy &> /dev/null; then
    echo "📦 Installing clippy..."
    rustup component add clippy
fi

if [ "$AUTO_FIX" = true ]; then
    echo "🔧 Auto-fixing code formatting..."
    cargo fmt --all
    echo "✅ Code formatting fixed!"
else
    echo "🔍 Checking code formatting..."
    if ! cargo fmt --all -- --check; then
        echo "❌ Code formatting check failed!"
        echo "💡 Run 'cargo fmt --all' to fix formatting issues"
        echo "💡 Or run this script with '--fix' flag to auto-fix formatting"
        exit 1
    fi
    echo "🔧 Code formatting is correct!"
fi
echo "🧹 Running cargo check to verify build..."
cargo check --all-targets --all-features

echo "🔍 Running clippy for linting (GitHub CI mode)..."
cargo clippy --all-targets --all-features -- -D warnings

echo "✅ Formatting and linting complete!"
echo ""
echo "📋 Summary:"
if [ "$AUTO_FIX" = true ]; then
    echo "  - Code formatting auto-fixed with rustfmt"
else
    echo "  - Code formatting verified with rustfmt (all files)"
fi
echo "  - Build verified with cargo check (all targets & features)"
echo "  - Code linted with clippy (GitHub CI mode)"
echo ""
echo "🎯 GitHub CI Compatibility:"
echo "  - Uses same clippy settings as GitHub Actions"
echo "  - Checks all targets and features"
echo "  - Treats warnings as errors (-D warnings)"
echo ""
echo "💡 To format automatically on save, consider:"
echo "  - Installing rust-analyzer in your IDE"
echo "  - Setting up pre-commit hooks"
echo "  - Using 'cargo fmt --check' in CI/CD"
echo "  - Using 'cargo clippy --all-targets --all-features -- -D warnings' in CI/CD" 