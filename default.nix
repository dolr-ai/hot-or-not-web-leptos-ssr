{}:
let
  rev = "33c968004c363ed5cddaba71215fd28845f68705";
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
  ] ++ (if pkgs.stdenv.isDarwin then [
      darwin.apple_sdk.frameworks.Foundation
      darwin.apple_sdk.frameworks.Security
      pkgs.darwin.libiconv
    ] else []);
  shellHook = ''
    export LLVM_PATH=$(which llvm)
    export RUSTC_WRAPPER=""
    export CC_wasm32_unknown_unknown=$LLVM_PATH/bin/clang
    export CXX_wasm32_unknown_unknown=$LLVM_PATH/bin/clang++
    export AS_wasm32_unknown_unknown=$LLVM_PATH/bin/llvm-as
    export AR_wasm32_unknown_unknown=$LLVM_PATH/bin/llvm-ar
    export STRIP_wasm32_unknown_unknown=$LLVM_PATH/bin/llvm-strip
  '';
  
}
