#!/bin/bash

# macOS Binary Signature Verification Script
# This script helps verify that TerminalAI binaries are properly signed and notarized

set -e

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
    echo "TerminalAI macOS Signature Verification Script"
    echo ""
    echo "Usage: $0 [options] [binary_paths...]"
    echo ""
    echo "Options:"
    echo "  --all              Verify all TerminalAI binaries in current directory"
    echo "  --gatekeeper       Also check Gatekeeper status"
    echo "  --verbose          Show detailed signature information"
    echo "  -h, --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --all                                    # Verify all binaries in current dir"
    echo "  $0 ./tai ./cp_ai                            # Verify specific binaries"
    echo "  $0 --all --gatekeeper --verbose            # Full verification with details"
    echo ""
}

# Parse command line arguments
VERIFY_ALL=false
CHECK_GATEKEEPER=false
VERBOSE=false
BINARIES=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            VERIFY_ALL=true
            shift
            ;;
        --gatekeeper)
            CHECK_GATEKEEPER=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        -*)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            BINARIES+=("$1")
            shift
            ;;
    esac
done

# Check if we're on macOS
if [[ "$(uname -s)" != "Darwin" ]]; then
    log_error "This script only works on macOS"
    exit 1
fi

# Function to verify a single binary
verify_binary() {
    local binary_path="$1"
    local binary_name=$(basename "$binary_path")
    
    log_info "Verifying $binary_name..."
    
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found: $binary_path"
        return 1
    fi
    
    if [ ! -x "$binary_path" ]; then
        log_error "Binary is not executable: $binary_path"
        return 1
    fi
    
    # Check code signature
    log_info "  Checking code signature..."
    if codesign --verify --verbose "$binary_path" 2>/dev/null; then
        log_success "  Code signature is valid"
        
        if [ "$VERBOSE" = true ]; then
            echo "    Signature details:"
            codesign -dv "$binary_path" 2>&1 | grep -E "(Authority=|TeamIdentifier=|Timestamp=)" | sed 's/^/      /'
        fi
    else
        log_error "  Code signature verification failed"
        return 1
    fi
    
    # Check notarization
    log_info "  Checking notarization status..."
    if xcrun stapler validate "$binary_path" 2>/dev/null; then
        log_success "  Binary is notarized"
    else
        log_warning "  Binary is not notarized (this is normal for development builds)"
    fi
    
    # Check Gatekeeper status
    if [ "$CHECK_GATEKEEPER" = true ]; then
        log_info "  Checking Gatekeeper status..."
        if spctl --assess --verbose "$binary_path" 2>/dev/null; then
            log_success "  Gatekeeper allows execution"
        else
            log_warning "  Gatekeeper may block execution (try: sudo spctl --master-disable)"
        fi
    fi
    
    echo ""
}

# Main verification logic
if [ "$VERIFY_ALL" = true ]; then
    log_info "Verifying all TerminalAI binaries in current directory..."
    
        # Find all TerminalAI binaries
    TAI_BINARIES=()
    for binary in tai; do
      if [ -f "$binary" ]; then
        TAI_BINARIES+=("$binary")
      fi
    done
    
    if [ ${#TAI_BINARIES[@]} -eq 0 ]; then
        log_error "No TerminalAI binaries found in current directory"
        log_info "Make sure you're in the directory containing the binaries"
        exit 1
    fi
    
    log_info "Found ${#TAI_BINARIES[@]} binaries to verify"
    
    for binary in "${TAI_BINARIES[@]}"; do
        verify_binary "$binary"
    done
    
elif [ ${#BINARIES[@]} -gt 0 ]; then
    log_info "Verifying specified binaries..."
    
    for binary in "${BINARIES[@]}"; do
        verify_binary "$binary"
    done
    
else
    log_error "No binaries specified"
    log_info "Use --all to verify all binaries in current directory"
    log_info "Or specify binary paths as arguments"
    show_help
    exit 1
fi

# Summary
log_success "Verification complete!"
log_info ""
log_info "If you see any warnings:"
log_info "  - Notarization warnings are normal for development builds"
log_info "  - Gatekeeper warnings can be resolved by running: sudo spctl --master-disable"
log_info "  - For production releases, all binaries should be signed and notarized" 