{ pkgs, inputs, ... }:

{
  packages =
    with pkgs;
    [
      protobuf
      diesel-cli
      cargo-udeps
      cargo-expand
      mold
      postgresql.lib
      atlas
    ]
    ++ lib.optionals pkgs.stdenv.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.CoreFoundation
      pkgs.darwin.apple_sdk.frameworks.Security
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  languages.rust = {
    enable = true;
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
    toolchain = {
      rustfmt = inputs.fenix.packages.${pkgs.system}.latest.rustfmt;
      rust-analyzer = inputs.fenix.packages.${pkgs.system}.latest.rust-analyzer;
    };
  };

  services.postgres = {
    enable = true;
    package = pkgs.postgresql_15.withPackages (p: [ p.postgis ]);
    listen_addresses = "127.0.0.1";
    settings = {
      max_wal_size = "10GB";
    };
  };

  dotenv.enable = true;

  # debug logging
  env.LOG_DATABASE = 1;
  env.ATLAS_NO_ANON_TELEMETRY = true;

  #  git-hooks.hooks = {
  #    clippy.enable = false;
  #  };

  enterShell = ''
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo "RUST_SRC_PATH: $RUST_SRC_PATH"
  '';
}
