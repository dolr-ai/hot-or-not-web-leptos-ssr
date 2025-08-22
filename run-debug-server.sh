#!/bin/bash

echo "Starting server with debug build..."
echo "Server will be available at http://localhost:3000"
echo "Test page will be at: http://localhost:3000/test-sentry"
echo ""
echo "Press Ctrl+C to stop the server"

LEPTOS_SITE_ROOT="target/site" \
LEPTOS_HASH_FILES=true \
./target/debug/hot-or-not-web-leptos-ssr