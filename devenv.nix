{ pkgs, ... }:

{
  env.RUST_LOG = "debug";

  packages = [ pkgs.rust-analyzer ];
  languages.rust.enable = true;

  #services.postgres.enable = true;
  #services.postgres.package = pkgs.postgresql_15;
  #services.postgres.listen_addresses = "127.0.0.1";
}
