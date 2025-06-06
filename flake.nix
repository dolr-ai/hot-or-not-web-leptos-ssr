{
    description = "A basic flake providing a shell with rustup";
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = {self, nixpkgs, flake-utils}: 
        flake-utils.lib.eachDefaultSystem (system: 
            let 
                pkgs = import nixpkgs {
                    inherit system;
                };    
                in
                {
                    devShells.default = pkgs.mkShell {
                        buildInputs = with pkgs; [
                            rustup
                            curl
                            openssl
                            binaryen
                            flyctl
                            leptosfmt
                            nodejs_22
                            rustup
                            openssl
                            git
                            cargo-leptos
                            protobuf_21
                            mold
                            tailwindcss_4
                            pkg-config
                        ];
                        shellHook = ''
                                export LD_LIBRARY_PATH=${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH
                                git submodule update --init --recursive
                                COOKIE_KEY=1267b291500365c42043e04bc69cf24a31495bd8936fc8d6794283675e288fad755971922d45cf1ca0b438df4fc847f39cb0b2aceb3a45673eff231cddb88dc9
                        '';
                    };
                }
        );
}