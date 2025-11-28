{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    rec {
      packages.${system} = rec {
        # build the admin frontend web assembly
        admin = pkgs.rustPlatform.buildRustPackage {
          name = "web";
          pname = "web";
          src = ./.;
          nativeBuildInputs = with pkgs; [
            dioxus-cli
            wasm-bindgen-cli
            binaryen
            llvmPackages_20.bintools
          ];
          buildInputs = with pkgs; [
            postgresql.lib
          ];

          buildPhase = ''
            dx bundle --frozen --release --platform web --package web --out-dir target/web-dist
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
