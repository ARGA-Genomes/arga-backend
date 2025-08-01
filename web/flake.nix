{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    dioxus-alpha.url = "github:CathalMullan/nixpkgs/dioxus-cli-v0.7.0";
  };

  outputs =
    {
      self,
      nixpkgs,
      dioxus-alpha,
    }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      dioxus = import dioxus-alpha { inherit system; };
    in
    rec {
      packages.${system} = rec {
        # build the admin frontend web assembly
        admin = pkgs.rustPlatform.buildRustPackage {
          name = "web";
          pname = "web";
          src = ./.;
          nativeBuildInputs = with pkgs; [
            dioxus.dioxus-cli
            wasm-bindgen-cli
            binaryen
            llvmPackages_14.bintools
          ];
          buildInputs = with pkgs; [
            postgresql.lib
          ];

          buildPhase = ''
            dx bundle --frozen --release --platform web --package web --verbose --trace --out-dir target/web-dist
          '';
          installPhase = ''
            mkdir -p $out
            cp -r target/web-dist/* $out
          '';
          cargoLock.lockFile = ./Cargo.lock;
        };

        default = admin;
      };
    };
}
