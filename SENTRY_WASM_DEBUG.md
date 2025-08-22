# Sentry WASM Debugging Setup

## Current Situation

**cargo-leptos does NOT generate source maps** despite what the `--wasm-debug` flag documentation suggests. The flag only adds DWARF debug information, which is not useful for browser-based Sentry debugging.

## What We Have

1. **DWARF Debug Info**: When building with `--wasm-debug`, the WASM file contains DWARF debug sections (`.debug_*`)
2. **Large File Size**: Debug WASM is ~287MB vs ~22MB stripped
3. **No Source Maps**: No `.wasm.map` files are generated

## Why Source Maps Aren't Generated

- cargo-leptos uses `wasm-bindgen-cli-support` internally
- It only calls `.debug()` and `.keep_debug()` methods
- There's no source map generation in the current implementation
- The comment "Includes source maps" in cli.rs:52 is incorrect

## Current Workaround

Since we can't get proper source maps from cargo-leptos, we have to:

### 1. Build with Debug Symbols
```bash
cargo leptos build --wasm-debug --release --lib-features release-lib --bin-features release-bin
```

### 2. Split Debug and Production Files
```bash
# Run the process script to:
# - Copy WASM with debug symbols to target/site/debug/
# - Strip production WASM in target/site/pkg/
./process-wasm-debug.sh
```

### 3. Upload to Sentry as Release Artifacts
```bash
export SENTRY_AUTH_TOKEN='your-token'
./sentry-upload.sh
```

## What Sentry Can Do With This

- **Limited Stack Traces**: Sentry can show function names but not source code locations
- **No Source Context**: Can't show the actual Rust code where errors occurred
- **Function Names Only**: DWARF debug info provides function names in stack traces

## Better Alternatives

### Option 1: Use trunk instead of cargo-leptos
Trunk has better source map support for WASM projects.

### Option 2: Use wasm-pack
```bash
wasm-pack build --target web --debug
```
This generates proper source maps.

### Option 3: Patch cargo-leptos
Add source map generation to cargo-leptos by modifying `src/compile/front.rs`.

### Option 4: Manual wasm-bindgen
Run wasm-bindgen manually with custom flags after cargo build.

## Improving Error Context Without Source Maps

Since we don't have source maps, maximize debugging info by:

1. **Better Panic Messages**:
```rust
#[cfg(target_arch = "wasm32")]
panic::set_hook(Box::new(|info| {
    let location = info.location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown".to_string());
    
    let msg = format!("Panic at {}: {}", location, info);
    web_sys::console::error_1(&msg.into());
}));
```

2. **Sentry Context**:
```javascript
Sentry.init({
    dsn: "your-dsn",
    release: "your-release",
    beforeSend(event, hint) {
        // Add context for WASM errors
        if (hint.originalException?.stack?.includes('wasm')) {
            event.tags = {
                ...event.tags,
                wasm_debug: 'dwarf_only',
                source_maps: 'unavailable'
            };
        }
        return event;
    }
});
```

3. **Custom Error Boundaries**:
Wrap WASM calls in try-catch blocks and add context before sending to Sentry.

## Summary

- **No source maps from cargo-leptos** (limitation of the tool)
- **DWARF debug info helps** but isn't ideal for browser debugging
- **Upload as release artifacts** for best results with current setup
- **Consider alternative build tools** for proper source map support