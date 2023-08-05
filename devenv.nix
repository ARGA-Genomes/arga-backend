{ pkgs, ... }:

{
  packages = with pkgs; [
    rust-analyzer
    protobuf
    diesel-cli
    openssl
    cargo-udeps
  ];

  languages.rust.enable = true;

  services.postgres.enable = true;
  services.postgres.package = pkgs.postgresql_15.withPackages (p: [ p.postgis ]);
  services.postgres.listen_addresses = "127.0.0.1";
  services.postgres.settings = {
    max_wal_size = "10GB";
  };
}
