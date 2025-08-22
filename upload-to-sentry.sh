#!/bin/bash

# Upload WASM files with source maps to Sentry
set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Uploading WASM with Source Maps to Sentry ===${NC}"

# Check token
if [ -z "$SENTRY_AUTH_TOKEN" ]; then
    echo "Setting SENTRY_AUTH_TOKEN..."
    export SENTRY_AUTH_TOKEN='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'
fi

# Sentry config
export SENTRY_URL="https://sentry.yral.com"
export SENTRY_ORG="sentry"
export SENTRY_PROJECT="leptos-ssr-browser"
export SENTRY_RELEASE=$(git rev-parse --short HEAD 2>/dev/null || echo "local-$(date +%s)")

echo "Release: $SENTRY_RELEASE"

# Create release
echo -e "\n${YELLOW}Creating release...${NC}"
sentry-cli releases new "$SENTRY_RELEASE" --org "$SENTRY_ORG" --project "$SENTRY_PROJECT" || true

# Upload WASM and source maps
echo -e "\n${YELLOW}Uploading WASM files and source maps...${NC}"
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        
        # Upload WASM
        echo "Uploading WASM: $filename"
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            --org "$SENTRY_ORG" --project "$SENTRY_PROJECT" \
            "$wasm_file" "/pkg/$filename" || true
        
        # Upload source map if exists
        if [ -f "$wasm_file.map" ]; then
            echo "Uploading source map: $filename.map"
            sentry-cli releases files "$SENTRY_RELEASE" upload \
                --org "$SENTRY_ORG" --project "$SENTRY_PROJECT" \
                "$wasm_file.map" "/pkg/$filename.map" || true
        fi
    fi
done

# Upload JS files
echo -e "\n${YELLOW}Uploading JS files...${NC}"
for js_file in target/site/pkg/*.js; do
    if [ -f "$js_file" ]; then
        filename=$(basename "$js_file")
        echo "Uploading JS: $filename"
        sentry-cli releases files "$SENTRY_RELEASE" upload \
            --org "$SENTRY_ORG" --project "$SENTRY_PROJECT" \
            "$js_file" "/pkg/$filename" || true
    fi
done

# List uploaded files
echo -e "\n${YELLOW}Uploaded files:${NC}"
sentry-cli releases files "$SENTRY_RELEASE" list \
    --org "$SENTRY_ORG" --project "$SENTRY_PROJECT" || true

# Set commits if in git repo
if [ -d .git ]; then
    echo -e "\n${YELLOW}Setting commits...${NC}"
    sentry-cli releases set-commits "$SENTRY_RELEASE" \
        --org "$SENTRY_ORG" --project "$SENTRY_PROJECT" \
        --auto || true
fi

# Finalize
echo -e "\n${YELLOW}Finalizing release...${NC}"
sentry-cli releases finalize "$SENTRY_RELEASE" \
    --org "$SENTRY_ORG" --project "$SENTRY_PROJECT" || true

echo -e "\n${GREEN}=== Upload Complete ===${NC}"
echo -e "\n${YELLOW}Make sure your app initializes Sentry with:${NC}"
echo "  release: '$SENTRY_RELEASE'"
echo ""
echo "The source maps should now work in Sentry for debugging!"