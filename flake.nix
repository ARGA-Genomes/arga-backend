{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      naersk,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk' = pkgs.callPackage naersk { };
      in
      rec {
        # build the backend executable
        packages.backend = naersk'.buildPackage {
          name = "arga-backend";
          pname = "arga-backend";
          src = ./.;
          nativeBuildInputs = [ pkgs.postgresql ];
        };

        packages.oci = pkgs.dockerTools.buildLayeredImage {
          name = "arga-backend";
          tag = "latest";

          contents = [ packages.backend ];

          config = {
            WorkingDir = "/";
            Env = [
              "BIND_ADDRESS=0.0.0.0:5000"
              "FRONTEND_URL=http://localhost:3000"
              "DATABASE_URL=postgres://arga@localhost/arga"
            ];
            ExposedPorts = {
              "5000/tcp" = { };
            };
            Cmd = [ "/bin/arga-backend" ];
            Labels = {
              "org.opencontainers.image.source" = "https://github.com/ARGA-Genomes/arga-backend";
              "org.opencontainers.image.url" = "https://github.com/ARGA-Genomes/arga-backend";
              "org.opencontainers.image.description" = "A container image running the backend server";
              "org.opencontainers.image.licenses" = "AGPL-3.0-or-later";
              "org.opencontainers.image.authors" = "ARGA Team <support@arga.org.au>";
            };
            Volumes = {
              "/.index" = { };
            };
          };
        };

        defaultPackage = packages.backend;
      }
    );
}
