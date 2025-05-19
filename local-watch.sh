#!/bin/bash

LEPTOS_SITE_ROOT="target/site" LEPTOS_HASH_FILES=true LEPTOS_TAILWIND_VERSION=v3.4.17 cargo leptos watch --bin-features local-bin --lib-features local-lib
