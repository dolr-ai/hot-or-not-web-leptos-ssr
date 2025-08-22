#!/bin/bash

# Script to prepare WASM files for Sentry browser debugging
# Since cargo-leptos doesn't generate source maps, we'll work with what we have

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Preparing WASM for Sentry Browser Debugging ===${NC}"

# Check for WASM files
if [ ! -d "target/site/pkg" ] && [ ! -d "/workspaces/hot-or-not-web-leptos-ssr/target/site/pkg" ]; then
    echo -e "${RED}Error: No target/site/pkg directory found${NC}"
    echo "Run 'cargo leptos build --wasm-debug --release' first"
    exit 1
fi

# Set working directory
cd /workspaces/hot-or-not-web-leptos-ssr

echo -e "\n${BLUE}Understanding the Situation:${NC}"
echo "• cargo-leptos doesn't generate source maps (.wasm.map files)"
echo "• The --wasm-debug flag adds DWARF debug info, not source maps"
echo "• Sentry browser SDK needs source maps or properly uploaded release artifacts"
echo "• DWARF debug info in WASM is primarily for native debuggers, not browser tools"

echo -e "\n${YELLOW}Current WASM Files:${NC}"
for wasm_file in target/site/pkg/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        size=$(ls -lh "$wasm_file" | awk '{print $5}')
        echo "• $filename ($size)"
        
        # Check for debug sections
        if wasm-objdump -h "$wasm_file" 2>/dev/null | grep -q "\.debug"; then
            echo "  ✓ Has DWARF debug sections"
        else
            echo "  ✗ No DWARF debug sections"
        fi
    fi
done

echo -e "\n${YELLOW}Sentry Integration Options:${NC}"
echo -e "${GREEN}Option 1: Upload as Release Artifacts${NC}"
echo "This is what we can do right now:"
cat << 'EOF'

export SENTRY_AUTH_TOKEN='your-token'
export SENTRY_URL="https://sentry.yral.com"
export SENTRY_ORG="sentry"
export SENTRY_PROJECT="leptos-ssr-browser"

# Create release
RELEASE=$(git rev-parse --short HEAD)
sentry-cli releases new "$RELEASE"

# Upload WASM and JS files
for file in target/site/pkg/*.{wasm,js}; do
    [ -f "$file" ] && sentry-cli releases files "$RELEASE" upload "$file" "/pkg/$(basename "$file")"
done

# Finalize
sentry-cli releases finalize "$RELEASE"
EOF

echo -e "\n${GREEN}Option 2: Use a Different Build Tool${NC}"
echo "For true source map support, consider:"
echo "• wasm-pack (generates source maps)"
echo "• trunk (has source map support)"
echo "• Manual wasm-bindgen with custom flags"

echo -e "\n${GREEN}Option 3: Enhance Error Context${NC}"
echo "Without source maps, improve debugging by:"
echo "• Adding more context to error messages"
echo "• Using panic hooks that capture stack traces"
echo "• Implementing custom error boundaries"

echo -e "\n${YELLOW}Recommended Approach for Now:${NC}"
echo "1. Keep WASM files with debug symbols (from --wasm-debug)"
echo "2. Upload them as release artifacts to Sentry"
echo "3. Configure your app to send detailed error context:"

cat << 'EOF'

// In your Rust code, add panic hook:
#[cfg(target_arch = "wasm32")]
panic::set_hook(Box::new(|info| {
    let msg = info.to_string();
    // Send to Sentry with context
    web_sys::console::error_1(&msg.into());
}));

// In your JS initialization:
Sentry.init({
    dsn: "your-dsn",
    release: "your-release",
    beforeSend(event, hint) {
        // Add WASM context if available
        if (hint.originalException?.stack?.includes('wasm')) {
            event.contexts = {
                ...event.contexts,
                wasm: {
                    type: 'wasm',
                    debug_symbols: 'available',
                    build_mode: 'release-with-debug'
                }
            };
        }
        return event;
    }
});
EOF

echo -e "\n${BLUE}Alternative: Generate Source Maps Manually${NC}"
echo "If you really need source maps, you could:"
echo "1. Use wasm2wat to convert WASM to WAT (text format)"
echo "2. Create a mapping file manually"
echo "3. Use specialized tools like 'wasm-sourcemap' (if available)"

echo -e "\n${GREEN}=== Preparation Complete ===${NC}"
echo -e "${YELLOW}Next step:${NC} Run ./sentry-upload.sh to upload files to Sentry"