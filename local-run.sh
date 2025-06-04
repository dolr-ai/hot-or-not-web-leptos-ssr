#!/bin/bash

# Check cargo-leptos version
CARGO_LEPTOS_VERSION=$(cargo leptos --version | cut -d' ' -f2)
if [ "$(printf '%s\n' "0.2.33" "$CARGO_LEPTOS_VERSION" | sort -V | head -n1)" = "$CARGO_LEPTOS_VERSION" ]; then
    # Version is less than 0.2.33, add RUSTFLAGS
    RUSTFLAGS="--cfg=erase_components" cargo leptos build --bin-features local-bin --lib-features local-lib || exit 1
    RUSTFLAGS="--cfg=erase_components" LEPTOS_SITE_ROOT="target/site" LEPTOS_HASH_FILES=true ./target/debug/hot-or-not-web-leptos-ssr
else
    # Version is 0.2.33 or higher
    cargo leptos build --bin-features local-bin --lib-features local-lib || exit 1
    LEPTOS_SITE_ROOT="target/site" LEPTOS_HASH_FILES=true ./target/debug/hot-or-not-web-leptos-ssr
fi
