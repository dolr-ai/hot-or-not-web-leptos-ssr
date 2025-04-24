{}:
let
  rev = "efc8686dfc086e7cde15504bcae3e4951303486a";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  # nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/master.tar.gz";
  pkgs = import nixpkgs { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    binaryen
    flyctl
    leptosfmt
    nodejs_22
    nodePackages_latest.tailwindcss
    cargo-leptos
    rustup
    openssl
    git
    protobuf_21
    mold
  ] ++ (if pkgs.stdenv.isDarwin then [
      darwin.apple_sdk.frameworks.Foundation
      darwin.apple_sdk.frameworks.Security
      pkgs.darwin.libiconv
    ] else []);
  shellHook = ''
    if [ -d "/opt/homebrew/opt/llvm" ]; then
      export LLVM_PATH="/opt/homebrew/opt/llvm"
    else
      export LLVM_PATH="$(which llvm)"
    fi
    export RUSTC_WRAPPER=""
    export CC_wasm32_unknown_unknown=$LLVM_PATH/bin/clang
    export CXX_wasm32_unknown_unknown=$LLVM_PATH/bin/clang++
    export AS_wasm32_unknown_unknown=$LLVM_PATH/bin/llvm-as
    export AR_wasm32_unknown_unknown=$LLVM_PATH/bin/llvm-ar
    export STRIP_wasm32_unknown_unknown=$LLVM_PATH/bin/llvm-strip
  '';
  
}
