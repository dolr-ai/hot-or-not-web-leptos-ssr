#!/bin/bash

echo "Building with WASM debug symbols enabled..."

# Clean previous builds to ensure fresh debug symbols
rm -rf ssr/target/site/pkg

# Build with debug symbols
cargo leptos build --wasm-debug || exit 1

echo "Build complete. Checking for generated files..."

# Check if sourcemaps were generated
echo "Contents of pkg directory:"
ls -la ssr/target/site/pkg/

echo ""
echo "Checking for .wasm and potential debug files:"
find ssr/target/site/pkg -name "*.wasm" -o -name "*.map" -o -name "*.js" | head -20

echo ""
echo "To run the server with debug build:"
echo "LEPTOS_SITE_ROOT=\"ssr/target/site\" LEPTOS_HASH_FILES=true ./ssr/target/debug/hot-or-not-web-leptos-ssr"