#!/bin/bash

# Check cargo-leptos version
CARGO_LEPTOS_VERSION=$(cargo leptos --version | cut -d' ' -f2)
if [ "$(printf '%s\n' "0.2.33" "$CARGO_LEPTOS_VERSION" | sort -V | head -n1)" = "$CARGO_LEPTOS_VERSION" ]; then
    # Version is less than 0.2.33, add RUSTFLAGS
    RUSTFLAGS="--cfg=erase_components" LEPTOS_SITE_ROOT="target/site" LEPTOS_HASH_FILES=true cargo leptos watch --bin-features local-bin --lib-features local-lib
else
    # Version is 0.2.33 or higher
    LEPTOS_SITE_ROOT="target/site" LEPTOS_HASH_FILES=true cargo leptos watch --bin-features local-bin --lib-features local-lib
fi
