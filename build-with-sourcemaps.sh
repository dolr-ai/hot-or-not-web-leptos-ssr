#!/bin/bash

# Complete workflow to build WASM with source maps for Sentry
# This script:
# 1. Builds with debug symbols
# 2. Generates source maps
# 3. Creates optimized production WASM
# 4. Prepares everything for Sentry upload

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Complete WASM Build with Source Maps ===${NC}"

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000/pkg}"
BUILD_MODE="${1:-release}"  # Can be "release" or "debug"

# Step 1: Build with debug symbols
echo -e "\n${BLUE}Step 1: Building with debug symbols...${NC}"
if [ "$BUILD_MODE" = "release" ]; then
    echo "Building release mode with debug symbols..."
    cargo leptos build --wasm-debug --release --lib-features release-lib --bin-features release-bin
else
    echo "Building debug mode..."
    cargo leptos build --wasm-debug
fi

# Step 2: Create backup of debug WASM
echo -e "\n${BLUE}Step 2: Backing up debug WASM...${NC}"
mkdir -p target/site/debug
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        cp "$wasm_file" "target/site/debug/${filename%.wasm}.debug.wasm"
        echo "  Backed up: $filename"
    fi
done

# Step 3: Generate source maps
echo -e "\n${BLUE}Step 3: Generating source maps...${NC}"
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        echo "Processing: $filename"
        
        # Generate source map with cargo-wasm2map
        # Note: --patch modifies the WASM to include sourceMappingURL
        if cargo wasm2map "$wasm_file" \
            --patch \
            --base-url "$BASE_URL" 2>&1 | grep -v "^warning:"; then
            
            if [ -f "$wasm_file.map" ]; then
                map_size=$(ls -lh "$wasm_file.map" | awk '{print $5}')
                echo -e "  ${GREEN}✓ Source map: ${filename}.map ($map_size)${NC}"
            else
                echo -e "  ${YELLOW}⚠ Source map generation may have failed${NC}"
            fi
        else
            echo -e "  ${RED}✗ Failed to generate source map${NC}"
        fi
    fi
done

# Step 4: Create optimized production WASM (optional)
echo -e "\n${BLUE}Step 4: Creating optimized production WASM...${NC}"
mkdir -p target/site/production
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        
        # Copy original with source map reference
        cp "$wasm_file" "target/site/production/$filename"
        
        # Also copy source map
        if [ -f "$wasm_file.map" ]; then
            cp "$wasm_file.map" "target/site/production/$filename.map"
        fi
        
        # Create stripped version without debug info (smaller size)
        echo "Creating stripped version..."
        wasm-opt "$wasm_file" -o "target/site/production/${filename%.wasm}.stripped.wasm" \
            --strip-debug \
            --strip-producers \
            --strip-dwarf \
            -Oz \
            --enable-bulk-memory \
            --enable-nontrapping-float-to-int 2>/dev/null || true
        
        # Show sizes
        original_size=$(ls -lh "$wasm_file" | awk '{print $5}')
        if [ -f "target/site/production/${filename%.wasm}.stripped.wasm" ]; then
            stripped_size=$(ls -lh "target/site/production/${filename%.wasm}.stripped.wasm" | awk '{print $5}')
            echo -e "  Original (with debug): $original_size"
            echo -e "  Stripped (no debug): $stripped_size"
        fi
    fi
done

# Step 5: Summary
echo -e "\n${GREEN}=== Build Complete ===${NC}"
echo -e "\n${BLUE}Generated Files:${NC}"
echo "• Debug WASM: target/site/debug/*.debug.wasm (with full debug info)"
echo "• Source Maps: target/site/pkg/*.wasm.map"
echo "• Production WASM: target/site/pkg/*.wasm (with sourceMappingURL)"
echo "• Stripped WASM: target/site/production/*.stripped.wasm (smallest size)"

echo -e "\n${YELLOW}For Sentry Upload:${NC}"
cat << EOF
# Set your Sentry token
export SENTRY_AUTH_TOKEN='your-token-here'
export SENTRY_RELEASE=\$(git rev-parse --short HEAD)

# Upload WASM and source maps
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "\$wasm_file.map" ]; then
        filename=\$(basename "\$wasm_file")
        
        # Upload WASM file
        sentry-cli releases files "\$SENTRY_RELEASE" upload \\
            --org sentry --project leptos-ssr-browser \\
            "\$wasm_file" "/pkg/\$filename"
        
        # Upload source map
        sentry-cli releases files "\$SENTRY_RELEASE" upload \\
            --org sentry --project leptos-ssr-browser \\
            "\$wasm_file.map" "/pkg/\$filename.map"
    fi
done

# Finalize release
sentry-cli releases finalize "\$SENTRY_RELEASE" \\
    --org sentry --project leptos-ssr-browser
EOF

echo -e "\n${BLUE}Server Configuration:${NC}"
echo "Ensure your server serves .wasm.map files with:"
echo "  Content-Type: application/json"
echo "  Access-Control-Allow-Origin: *"

echo -e "\n${GREEN}✓ Ready for deployment with source map support!${NC}"