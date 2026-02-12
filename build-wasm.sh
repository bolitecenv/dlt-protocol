#!/bin/bash

# Build script for WASM examples
set -e

echo "🦀 Building DLT Protocol for WASM..."

# Build the WASM demo example
cargo build --target wasm32-unknown-unknown --example wasm_demo --release --features std

echo "✅ WASM build complete!"
echo ""
echo "📦 WASM file location:"
echo "   target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm"
echo ""
echo "🌐 To test in browser:"
echo "   1. Start a local server: python3 -m http.server 8000"
echo "   2. Open: http://localhost:8000/examples/wasm_test.html"
echo ""
