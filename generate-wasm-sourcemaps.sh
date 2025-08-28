#!/bin/bash

# Script to generate source maps for WASM files using cargo-wasm2map
# Run this after building with cargo-leptos

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== WASM Source Map Generation ===${NC}"

# Check if cargo-wasm2map is installed
if ! command -v cargo-wasm2map &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-wasm2map...${NC}"
    cargo install cargo-wasm2map
fi

# Base URL for source map references (adjust as needed)
BASE_URL="${BASE_URL:-http://localhost:3000/pkg}"

echo -e "\n${BLUE}Generating source maps for WASM files...${NC}"
echo "Base URL: $BASE_URL"

# Process all WASM files in target/site/pkg
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        echo -e "\n${YELLOW}Processing: $filename${NC}"
        
        # Check if WASM has debug info
        if wasm-objdump -h "$wasm_file" 2>/dev/null | grep -q "\.debug"; then
            echo "  ✓ Has DWARF debug sections"
            
            # Generate source map with cargo-wasm2map
            # -patch: patches the WASM file to include sourceMappingURL
            # -base-url: sets the base URL for source references
            echo "  Generating source map..."
            
            if cargo wasm2map "$wasm_file" \
                --patch \
                --base-url "$BASE_URL" 2>/dev/null; then
                
                # Check if .map file was created
                if [ -f "$wasm_file.map" ]; then
                    map_size=$(ls -lh "$wasm_file.map" | awk '{print $5}')
                    echo -e "  ${GREEN}✓ Source map generated: ${filename}.map ($map_size)${NC}"
                    
                    # Verify the WASM was patched with sourceMappingURL
                    if strings "$wasm_file" 2>/dev/null | grep -q "sourceMappingURL"; then
                        echo -e "  ${GREEN}✓ WASM patched with sourceMappingURL${NC}"
                    else
                        echo -e "  ${YELLOW}⚠ sourceMappingURL not found in WASM${NC}"
                    fi
                else
                    echo -e "  ${RED}✗ Source map file not created${NC}"
                fi
            else
                echo -e "  ${RED}✗ Failed to generate source map${NC}"
                echo "  Trying without --patch flag..."
                
                # Try without patching (just generate the .map file)
                if cargo wasm2map "$wasm_file" --base-url "$BASE_URL" 2>/dev/null; then
                    if [ -f "$wasm_file.map" ]; then
                        echo -e "  ${GREEN}✓ Source map generated (without patching)${NC}"
                        echo -e "  ${YELLOW}Note: You'll need to manually add sourceMappingURL to your HTML/JS${NC}"
                    fi
                fi
            fi
        else
            echo -e "  ${YELLOW}✗ No DWARF debug sections found${NC}"
            echo "  Build with: cargo leptos build --wasm-debug"
        fi
    fi
done

echo -e "\n${BLUE}Checking generated source maps...${NC}"
ls -lh target/site/pkg/*.map 2>/dev/null || echo "No source maps found"

echo -e "\n${GREEN}=== Source Map Generation Complete ===${NC}"

echo -e "\n${YELLOW}Next Steps:${NC}"
echo "1. Ensure your web server serves .map files with correct MIME type:"
echo "   Content-Type: application/json"
echo ""
echo "2. Upload both .wasm and .wasm.map files to Sentry:"
echo "   ./sentry-upload.sh"
echo ""
echo "3. In your HTML/JS, the WASM should now reference its source map"
echo ""
echo "4. For development, ensure your server is running at: $BASE_URL"

echo -e "\n${BLUE}Sentry Upload Commands:${NC}"
cat << 'EOF'
export SENTRY_AUTH_TOKEN='your-token'
export SENTRY_RELEASE=$(git rev-parse --short HEAD)

# Upload WASM with source maps
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file.map" ]; then
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            "$wasm_file" "/pkg/$(basename $wasm_file)"
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            "$wasm_file.map" "/pkg/$(basename $wasm_file).map"
    fi
done
EOF