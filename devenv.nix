{ pkgs, inputs, ... }:

{
  packages = with pkgs; [
    protobuf
    diesel-cli
    cargo-udeps
    cargo-expand
    mold
    postgresql.lib
  ] ++ lib.optionals pkgs.stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.CoreFoundation
    pkgs.darwin.apple_sdk.frameworks.Security
    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  languages.rust = {
    enable = true;
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
    toolchain = {
      rustfmt = inputs.fenix.packages.${pkgs.system}.latest.rustfmt;
    };
  };

  services.postgres = {
    enable = true;
    package = pkgs.postgresql_15.withPackages (p: [ p.postgis ]);
    listen_addresses = "127.0.0.1";
    settings = {
      max_wal_size = "10GB";
      log_connections = true;
      log_statement = "all";
      logging_collector = true;
      log_disconnections = true;
    };
  };

  dotenv.disableHint = true;

  # debug logging
  env.LOG_DATABASE = 1;

  pre-commit.hooks = {
    clippy.enable = false;
  };
}
