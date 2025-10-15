# Install cargo binstall
curl -L https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz -o cargo-binstall.tgz;
tar -xzf cargo-binstall.tgz;
mkdir -p ~/.local/bin;
mv ./cargo-binstall ~/.local/bin/cargo-binstall;
chmod +x ~/.local/bin/cargo-binstall;
rm cargo-binstall.tgz;

# Install cargo-leptos with cargo binstall
cargo binstall cargo-leptos --no-confirm;

# Install leptosfmt using cargo binstall
cargo binstall leptosfmt --no-confirm;

# Fetch git submodules for ml-feed protobuf contracts
git submodule update --init --recursive;

# Enable the env file
cp .env.example .env;