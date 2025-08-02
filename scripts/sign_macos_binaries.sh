#!/bin/bash

# Local macOS code signing script for TerminalAI
# This script signs binaries built locally with your Developer ID certificate

set -e

# Configuration
BINARY_DIR="target/release"
RELEASE_DIR="release"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions for colored output
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Help function
show_help() {
    echo "TerminalAI macOS Code Signing Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --cert-name NAME    Developer ID certificate name (optional)"
    echo "  --binary-dir DIR    Directory containing binaries (default: target/release)"
    echo "  --output-dir DIR    Output directory for signed binaries (default: release)"
    echo "  --verify-only       Only verify existing signatures"
    echo "  --list-certs        List available code signing certificates"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Auto-detect certificate and sign"
    echo "  $0 --cert-name 'Developer ID Application: John Doe'"
    echo "  $0 --verify-only                     # Just verify existing signatures"
    echo "  $0 --list-certs                      # List available certificates"
    echo ""
    echo "Prerequisites:"
    echo "  - macOS with Xcode command line tools"
    echo "  - Valid code signing certificate in Keychain"
    echo "    (Developer ID Application for distribution, or Apple Development for local use)"
    echo "  - Binaries built with 'cargo build --release'"
}

# Parse command line arguments
CERT_NAME=""
VERIFY_ONLY=false
LIST_CERTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --cert-name)
            CERT_NAME="$2"
            shift 2
            ;;
        --binary-dir)
            BINARY_DIR="$2"
            shift 2
            ;;
        --output-dir)
            RELEASE_DIR="$2"
            shift 2
            ;;
        --verify-only)
            VERIFY_ONLY=true
            shift
            ;;
        --list-certs)
            LIST_CERTS=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Change to project root
cd "$PROJECT_ROOT"

# List certificates and exit if requested
if [ "$LIST_CERTS" = true ]; then
    log_info "Available code signing certificates:"
    security find-identity -v -p codesigning
    exit 0
fi

# Check if we're on macOS
if [[ "$(uname -s)" != "Darwin" ]]; then
    log_error "This script only works on macOS"
    exit 1
fi

# Define binaries to sign
BINARIES=(
    "tai"
)

# Verify-only mode
if [ "$VERIFY_ONLY" = true ]; then
    log_info "Verifying code signatures..."
    
    for binary in "${BINARIES[@]}"; do
        binary_path="$BINARY_DIR/$binary"
        if [ -f "$binary_path" ]; then
            log_info "Verifying $binary..."
            if codesign --verify --verbose "$binary_path" 2>/dev/null; then
                log_success "$binary is properly signed"
                # Show signature details
                AUTHORITY=$(codesign -dv "$binary_path" 2>&1 | grep "Authority=" || true)
                TEAM_ID=$(codesign -dv "$binary_path" 2>&1 | grep "TeamIdentifier=" || true)
                if [ -n "$AUTHORITY" ]; then
                    echo "    $AUTHORITY"
                elif [ -n "$TEAM_ID" ]; then
                    echo "    $TEAM_ID"
                fi
            else
                log_warning "$binary is not signed or signature is invalid"
            fi
        else
            log_warning "Binary not found: $binary_path"
        fi
    done
    exit 0
fi

# Check if binary directory exists
if [ ! -d "$BINARY_DIR" ]; then
    log_error "Binary directory not found: $BINARY_DIR"
    log_info "Please run 'cargo build --release' first"
    exit 1
fi

# Auto-detect certificate if not specified
if [ -z "$CERT_NAME" ]; then
    log_info "Auto-detecting code signing certificate..."
    
    # First try to find Developer ID Application certificate (for distribution)
    CERT_NAME=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | grep -o '"[^"]*"' | sed 's/"//g')
    
    # If no Developer ID Application found, fall back to Apple Development (for local development)
    if [ -z "$CERT_NAME" ]; then
        log_warning "No Developer ID Application certificate found, trying Apple Development certificates..."
        CERT_NAME=$(security find-identity -v -p codesigning | grep "Apple Development" | head -1 | grep -o '"[^"]*"' | sed 's/"//g')
    fi
    
    if [ -z "$CERT_NAME" ]; then
        log_error "No suitable code signing certificate found"
        log_info "Available certificates:"
        security find-identity -v -p codesigning
        log_info ""
        log_info "To install a certificate:"
        log_info "1. Double-click your .cer file to add it to Keychain"
        log_info "2. Or use: security import certificate.p12 -k ~/Library/Keychains/login.keychain"
        exit 1
    fi
    
    log_success "Found certificate: $CERT_NAME"
fi

# Create output directory
mkdir -p "$RELEASE_DIR"

# Sign binaries
log_info "Signing binaries with certificate: $CERT_NAME"

for binary in "${BINARIES[@]}"; do
    binary_path="$BINARY_DIR/$binary"
    output_path="$RELEASE_DIR/$binary"
    
    if [ -f "$binary_path" ]; then
        log_info "Signing $binary..."
        
        # Copy binary to output directory
        cp "$binary_path" "$output_path"
        
        # Sign the binary
        if codesign --force --sign "$CERT_NAME" --timestamp --options runtime "$output_path"; then
            # Verify signature
            if codesign --verify --verbose "$output_path" 2>/dev/null; then
                log_success "$binary signed successfully"
            else
                log_error "Failed to verify signature for $binary"
                exit 1
            fi
        else
            log_error "Failed to sign $binary"
            exit 1
        fi
    else
        log_warning "Binary not found: $binary_path"
    fi
done

# Copy config file if it exists
if [ -f "terminalai.conf" ]; then
    cp "terminalai.conf" "$RELEASE_DIR/"
    log_info "Copied terminalai.conf to $RELEASE_DIR/"
fi

log_success "All binaries signed successfully!"
log_info "Signed binaries are in: $RELEASE_DIR/"

# Show signature verification
echo ""
log_info "Signature verification:"
for binary in "${BINARIES[@]}"; do
    output_path="$RELEASE_DIR/$binary"
    if [ -f "$output_path" ]; then
        echo "  $binary:"
        AUTHORITY=$(codesign -dv "$output_path" 2>&1 | grep "Authority=" || true)
        TEAM_ID=$(codesign -dv "$output_path" 2>&1 | grep "TeamIdentifier=" || true)
        if [ -n "$AUTHORITY" ]; then
            echo "    $AUTHORITY"
        elif [ -n "$TEAM_ID" ]; then
            echo "    $TEAM_ID"
        fi
    fi
done

echo ""
log_info "Installation instructions:"
echo "  sudo cp $RELEASE_DIR/* /usr/local/bin/"
echo "  or: cp $RELEASE_DIR/* ~/.local/bin/"

echo ""
if [[ "$CERT_NAME" == *"Apple Development"* ]]; then
    log_warning "Note: Signed with Apple Development certificate (for local development)."
    log_info "For public distribution, you need a Developer ID Application certificate."
else
    log_warning "Note: Binaries are signed but not notarized."
    log_info "Users may still see Gatekeeper warnings on first run."
    log_info "For full compliance, submit binaries for notarization to Apple."
fi