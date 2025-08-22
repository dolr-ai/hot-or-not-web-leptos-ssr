#!/bin/bash

# Script to separate WASM debug symbols for Sentry
# Usage: ./process-wasm-debug.sh [--upload-sentry]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== WASM Debug Symbol Processing ===${NC}"

# Step 1: Create debug directory
echo -e "\n${YELLOW}Step 1: Creating debug directory...${NC}"
mkdir -p target/site/debug

# Step 2: Copy and strip WASM files
echo -e "\n${YELLOW}Step 2: Processing WASM files...${NC}"
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        echo -e "\nProcessing ${GREEN}$filename${NC}..."
        
        # Copy original with debug symbols for Sentry
        cp "$wasm_file" "target/site/debug/${filename%.wasm}.debug.wasm"
        
        # Show original size with debug symbols
        original_size=$(ls -lh "$wasm_file" | awk '{print $5}')
        echo "  Original size (with debug): $original_size"
        
        # Check wasm-opt version
        wasm_opt_version=$(/usr/local/cargo/bin/wasm-opt --version | grep -oE '[0-9]+' | head -1)
        echo "  Using wasm-opt version: $wasm_opt_version"
        
        
        # Strip debug symbols from production file
        # Try stripping with error handling
        if /usr/local/cargo/bin/wasm-opt "$wasm_file" -o "$wasm_file.tmp" \
            --strip-debug \
            --strip-producers \
            --strip-dwarf \
            -Oz \
            --enable-bulk-memory \
            --enable-nontrapping-float-to-int 2>/dev/null; then
            mv "$wasm_file.tmp" "$wasm_file"
        else
            echo -e "  ${YELLOW}Warning: wasm-opt failed to strip debug symbols${NC}"
            echo "  Trying alternative approach..."
            
            # Try with just basic stripping
            if /usr/local/cargo/bin/wasm-opt "$wasm_file" -o "$wasm_file.tmp" \
                --strip \
                -Oz 2>/dev/null; then
                mv "$wasm_file.tmp" "$wasm_file"
                echo -e "  ${GREEN}Successfully stripped with basic options${NC}"
            else
                echo -e "  ${RED}Error: Could not strip debug symbols${NC}"
                echo "  Production file kept with debug symbols (larger size)"
                echo "  Consider updating wasm-opt: npm install -g wasm-opt@latest"
            fi
        fi
        
        # Show stripped size
        stripped_size=$(ls -lh "$wasm_file" | awk '{print $5}')
        echo "  Stripped size (production): $stripped_size"
        
        # Calculate size reduction
        original_bytes=$(stat -c%s "target/site/debug/${filename%.wasm}.debug.wasm")
        stripped_bytes=$(stat -c%s "$wasm_file")
        reduction=$(( 100 - (stripped_bytes * 100 / original_bytes) ))
        echo -e "  ${GREEN}Size reduction: ${reduction}%${NC}"
    fi
done

# Step 3: Show results
echo -e "\n${YELLOW}Step 3: Results${NC}"
echo -e "\nDebug WASM files (for Sentry):"
ls -lh target/site/debug/*.debug.wasm 2>/dev/null || echo "No debug files found"

echo -e "\nProduction WASM files (stripped):"
ls -lh target/site/pkg/*.wasm 2>/dev/null || echo "No production files found"

# Step 4: Optional Sentry upload
if [ "$1" == "--upload-sentry" ]; then
    echo -e "\n${YELLOW}Step 4: Uploading to Sentry...${NC}"
    
    # Check if SENTRY_AUTH_TOKEN is set
    if [ -z "$SENTRY_AUTH_TOKEN" ]; then
        echo -e "${RED}Error: SENTRY_AUTH_TOKEN environment variable not set${NC}"
        echo "Please set it before running with --upload-sentry:"
        echo "export SENTRY_AUTH_TOKEN='your-token-here'"
        exit 1
    fi
    
    # Set Sentry configuration
    export SENTRY_URL="https://sentry.yral.com"
    export SENTRY_ORG="sentry"
    export SENTRY_PROJECT="leptos-ssr-browser"
    
    # Generate release version (you can customize this)
    SENTRY_RELEASE=$(git rev-parse --short HEAD 2>/dev/null || echo "local-$(date +%Y%m%d-%H%M%S)")
    echo "Using release: $SENTRY_RELEASE"
    
    # Upload debug WASM files with proper configuration
    for debug_wasm in target/site/debug/*.debug.wasm; do
        if [ -f "$debug_wasm" ]; then
            filename=$(basename "$debug_wasm")
            echo "Uploading: $filename"
            
            # Upload as WASM debug file with DWARF info
            sentry-cli debug-files upload \
                --org "$SENTRY_ORG" \
                --project "$SENTRY_PROJECT" \
                --auth-token "$SENTRY_AUTH_TOKEN" \
                --include-sources \
                --wait \
                "$debug_wasm" || echo "  Warning: Upload failed for $filename"
                
            # Also try uploading to releases artifacts
            echo "Uploading to release artifacts..."
            sentry-cli releases files "$SENTRY_RELEASE" upload \
                --org "$SENTRY_ORG" \
                --project "$SENTRY_PROJECT" \
                --auth-token "$SENTRY_AUTH_TOKEN" \
                "$debug_wasm" "~/${filename%.debug.wasm}.wasm" || echo "  Warning: Release upload failed"
        fi
    done
    
    # Upload JS files for context
    echo -e "\nUploading JS files..."
    for js_file in target/site/pkg/*.js; do
        if [ -f "$js_file" ]; then
            echo "Uploading: $(basename "$js_file")"
            sentry-cli debug-files upload \
                --org "$SENTRY_ORG" \
                --project "$SENTRY_PROJECT" \
                --auth-token "$SENTRY_AUTH_TOKEN" \
                "$js_file" || echo "  Warning: Upload failed for $(basename "$js_file")"
        fi
    done
    
    # Create and finalize release
    echo -e "\nCreating Sentry release..."
    sentry-cli releases new "$SENTRY_RELEASE" \
        --org "$SENTRY_ORG" \
        --project "$SENTRY_PROJECT" || echo "  Warning: Release creation failed"
    
    sentry-cli releases finalize "$SENTRY_RELEASE" \
        --org "$SENTRY_ORG" \
        --project "$SENTRY_PROJECT" || echo "  Warning: Release finalization failed"
    
    echo -e "${GREEN}Sentry upload complete!${NC}"
else
    echo -e "\n${YELLOW}Tip:${NC} To upload to Sentry, run:"
    echo "  export SENTRY_AUTH_TOKEN='your-token-here'"
    echo "  ./process-wasm-debug.sh --upload-sentry"
fi

echo -e "\n${GREEN}=== Processing Complete ===${NC}"