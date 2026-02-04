#!/usr/bin/env bash
# Build the moltis-wasm package for browser use
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_CRATE="$PROJECT_ROOT/crates/wasm"

# Default values
TARGET="web"
FEATURES=""
RELEASE=true

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --target <target>   Build target: web, nodejs, bundler (default: web)"
    echo "  --debug             Build without optimizations (for debugging)"
    echo "  --console-panic     Enable better panic messages in browser console"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                          # Build for web with release optimizations"
    echo "  $0 --target nodejs          # Build for Node.js"
    echo "  $0 --debug --console-panic  # Debug build with panic hook"
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            TARGET="$2"
            shift 2
            ;;
        --debug)
            RELEASE=false
            shift
            ;;
        --console-panic)
            FEATURES="console-panic"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed."
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Check for wasm32 target
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

echo "Building moltis-wasm..."
echo "  Target: $TARGET"
echo "  Release: $RELEASE"
echo "  Features: ${FEATURES:-none}"

cd "$PROJECT_ROOT"

# Build command
CMD="wasm-pack build crates/wasm --target $TARGET"

if [ "$RELEASE" = false ]; then
    CMD="$CMD --dev"
fi

if [ -n "$FEATURES" ]; then
    CMD="$CMD -- --features $FEATURES"
fi

echo ""
echo "Running: $CMD"
echo ""

eval "$CMD"

echo ""
echo "Build complete!"
echo "Output: $WASM_CRATE/pkg/"
echo ""
echo "Files:"
ls -la "$WASM_CRATE/pkg/" 2>/dev/null || echo "  (no output yet)"
