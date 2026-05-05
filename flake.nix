{
  description = "Rust development shell and package";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        wasm-bindgen-cli-0-2-120 = pkgs.rustPlatform.buildRustPackage rec {
          pname = "wasm-bindgen-cli";
          version = "0.2.120";

          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-Dkkx8Bhfk+y/jEz9Fzwytmv2N3Gj/7ST+5MlPRzzetU=";
          };

          cargoHash = "sha256-5Zu/Sh9aBMxB+KGC1MHWJAQ8PuE40M6lsenkpFEwJ6A=";
          doCheck = false;
        };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rust-app";
          version = "0.1.0";

          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.makeWrapper ];

          postFixup = ''
            for bin in "$out"/bin/*; do
              wrapProgram "$bin" --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.binaryen pkgs.tinymist pkgs.llvmPackages.lld wasm-bindgen-cli-0-2-120 pkgs.dioxus-cli ]}
            done
          '';

        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            binaryen
            llvmPackages.lld
            wasm-bindgen-cli-0-2-120
            dioxus-cli
            tinymist
          ];
        };
      }
    );
}
