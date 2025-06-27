# Install cargo binstall
http get https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz | save cargo-binstall.tgz;
tar -xzf cargo-binstall.tgz;
mkdir ~/.local/bin;
mv ./cargo-binstall ~/.local/bin/cargo-binstall;
chmod +x ~/.local/bin/cargo-binstall;
rm cargo-binstall.tgz;

# Install cargo-leptos with cargo binstall
# cargo binstall cargo-leptos --no-confirm;
cargo install --locked cargo-leptos --git https://github.com/saikatdas0790/cargo-leptos --branch saikatdas0790/fix-wasm-opt;

# Install leptosfmt using cargo binstall
cargo binstall leptosfmt --no-confirm;

# cargo leptos build --release
cargo leptos build --release --lib-features release-lib --bin-features release-bin