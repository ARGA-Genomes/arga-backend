{ pkgs, inputs, ... }:

{
  packages = with pkgs; [
    protobuf
    diesel-cli
    cargo-udeps
    mold
  ];

  languages.rust = {
    enable = true;
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
    toolchain = {
      rustfmt = inputs.fenix.packages.${pkgs.system}.latest.rustfmt;
    };
  };

  services.postgres.enable = true;
  services.postgres.package = pkgs.postgresql_15.withPackages (p: [ p.postgis ]);
  services.postgres.listen_addresses = "127.0.0.1";
  services.postgres.settings = {
    max_wal_size = "10GB";
  };

  dotenv.disableHint = true;

  pre-commit.hooks = {
    clippy.enable = false;
  };
}
