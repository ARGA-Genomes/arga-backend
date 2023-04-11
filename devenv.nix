{ pkgs, ... }:

{
  packages = with pkgs; [
    rust-analyzer
    protobuf
    diesel-cli
    openssl
  ];

  languages.rust.enable = true;

  services.postgres.enable = true;
  services.postgres.package = pkgs.postgresql_15;
  services.postgres.listen_addresses = "127.0.0.1";
}
