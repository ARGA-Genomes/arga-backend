{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    web = {
      url = "./web";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      web,
    }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      naersk' = pkgs.callPackage naersk { };
    in
    rec {
      packages.${system} = rec {
        # build the backend executable
        backend = naersk'.buildPackage {
          name = "arga-backend";
          pname = "arga-backend";
          src = ./.;
          nativeBuildInputs = [ pkgs.postgresql.lib ];
        };

        # build the admin frontend web assembly
        admin = web.packages.${system}.admin;

        # build the container image
        oci = pkgs.dockerTools.buildLayeredImage {
          name = "backend";
          tag = "latest";

          contents = [
            backend
            admin
          ];

          config = {
            WorkingDir = "/";
            Env = [
              "BIND_ADDRESS=0.0.0.0:5000"
              "FRONTEND_URL=http://arga.org.au"
              "ADMIN_PROXY=/public"
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

        # build the database migrator image
        migrator =
          let
            schema = pkgs.lib.fileset.toSource {
              root = ./core;
              fileset = ./core/schema.sql;
            };
            atlasConfig = pkgs.lib.fileset.toSource {
              root = ./core;
              fileset = ./core/atlas.hcl;
            };
            migrations = pkgs.lib.fileset.toSource {
              root = ./core;
              fileset = ./core/migrations;
            };
          in
          pkgs.dockerTools.buildLayeredImage {
            name = "backend-migrator";
            tag = "latest";

            contents = [
              pkgs.atlas
              atlasConfig
              schema
              migrations
            ];

            config = {
              WorkingDir = "/";
              Env = [
                "DATABASE_URL=postgres://arga@localhost/arga"
              ];
              Cmd = [
                "/bin/atlas"
                "migrate"
                "apply"
                "--env"
                "arga"
                "--baseline"
                "20250605060808"
              ];
              Labels = {
                "org.opencontainers.image.source" = "https://github.com/ARGA-Genomes/arga-backend";
                "org.opencontainers.image.url" = "https://github.com/ARGA-Genomes/arga-backend";
                "org.opencontainers.image.description" = "A container image for running migration jobs";
                "org.opencontainers.image.licenses" = "AGPL-3.0-or-later";
                "org.opencontainers.image.authors" = "ARGA Team <support@arga.org.au>";
              };
            };
          };

        default = backend;
      };
    };
}
