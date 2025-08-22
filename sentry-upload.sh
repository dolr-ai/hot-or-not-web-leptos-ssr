#!/bin/bash

# Script to upload WASM files to Sentry for browser debugging
# Usage: ./sentry-upload.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Sentry WASM Upload for Browser Debugging ===${NC}"

# Check if SENTRY_AUTH_TOKEN is set
if [ -z "$SENTRY_AUTH_TOKEN" ]; then
    echo -e "${RED}Error: SENTRY_AUTH_TOKEN environment variable not set${NC}"
    echo "Please set it before running:"
    echo "export SENTRY_AUTH_TOKEN='your-token-here'"
    exit 1
fi

# Set Sentry configuration
export SENTRY_URL="https://sentry.yral.com"
export SENTRY_ORG="sentry"
export SENTRY_PROJECT="leptos-ssr-browser"

# Generate release version
SENTRY_RELEASE=$(git rev-parse --short HEAD 2>/dev/null || echo "local-$(date +%Y%m%d-%H%M%S)")
echo "Using release: $SENTRY_RELEASE"

echo -e "\n${YELLOW}Creating Sentry release...${NC}"
sentry-cli releases new "$SENTRY_RELEASE" \
    --org "$SENTRY_ORG" \
    --project "$SENTRY_PROJECT" || echo "Release may already exist"

echo -e "\n${YELLOW}Uploading WASM and JS files as release artifacts...${NC}"

# Upload WASM files
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        echo "Uploading WASM: $filename ($(ls -lh "$wasm_file" | awk '{print $5}'))"
        
        # Upload as release artifact with proper path
        # The path should match how it's loaded in the browser
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            --org "$SENTRY_ORG" \
            --project "$SENTRY_PROJECT" \
            "$wasm_file" "/pkg/$filename" || echo "  Warning: Upload failed for $filename"
            
        # Also try with tilde prefix (common for web assets)
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            --org "$SENTRY_ORG" \
            --project "$SENTRY_PROJECT" \
            "$wasm_file" "~/pkg/$filename" || echo "  Warning: Upload with ~ failed"
    fi
done

# Upload JS files
for js_file in target/site/pkg/*.js; do
    if [ -f "$js_file" ]; then
        filename=$(basename "$js_file")
        echo "Uploading JS: $filename"
        
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            --org "$SENTRY_ORG" \
            --project "$SENTRY_PROJECT" \
            "$js_file" "/pkg/$filename" || echo "  Warning: Upload failed for $filename"
            
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            --org "$SENTRY_ORG" \
            --project "$SENTRY_PROJECT" \
            "$js_file" "~/pkg/$filename" || echo "  Warning: Upload with ~ failed"
    fi
done

# List uploaded files
echo -e "\n${YELLOW}Listing uploaded files for release...${NC}"
sentry-cli releases files "$SENTRY_RELEASE" list \
    --org "$SENTRY_ORG" \
    --project "$SENTRY_PROJECT" || echo "Could not list files"

# Set commits if in a git repo
if [ -d .git ]; then
    echo -e "\n${YELLOW}Setting commits for release...${NC}"
    sentry-cli releases set-commits "$SENTRY_RELEASE" \
        --org "$SENTRY_ORG" \
        --project "$SENTRY_PROJECT" \
        --auto || echo "Could not set commits"
fi

# Finalize release
echo -e "\n${YELLOW}Finalizing release...${NC}"
sentry-cli releases finalize "$SENTRY_RELEASE" \
    --org "$SENTRY_ORG" \
    --project "$SENTRY_PROJECT" || echo "Could not finalize"

echo -e "\n${GREEN}=== Upload Complete ===${NC}"
echo -e "\n${YELLOW}Important Notes:${NC}"
echo "1. Make sure your app initializes Sentry with release: '$SENTRY_RELEASE'"
echo "2. The file paths must match how they're served (e.g., /pkg/filename.wasm)"
echo "3. WASM debugging works best with source maps (build with WASM_BINDGEN_DEBUG=1)"
echo ""
echo "In your app's Sentry init:"
echo "  Sentry.init({"
echo "    dsn: 'your-dsn',"
echo "    release: '$SENTRY_RELEASE',"
echo "    integrations: [new Sentry.Integrations.Wasm()]"
echo "  });"