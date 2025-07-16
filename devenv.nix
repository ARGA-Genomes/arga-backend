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

      dioxus-cli
      wasm-bindgen-cli
      tailwindcss_4
    ]
    ++ lib.optionals pkgs.stdenv.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.CoreFoundation
      pkgs.darwin.apple_sdk.frameworks.Security
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [ "wasm32-unknown-unknown" ];
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
    toolchain = {
      rustfmt = inputs.fenix.packages.${pkgs.system}.latest.rustfmt;
      # rust-analyzer = inputs.fenix.packages.${pkgs.system}.latest.rust-analyzer;
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

  # dev proxy server to redirect requests between frontend and backend
  # servers. this allows us to avoid CORS and cookie issues since all
  # requests are sent to the same domain and port
  services.nginx = {
    enable = true;

    # upstream for the backend api and the admin web dev server
    # we also add websocket support to enable live reload
    httpConfig = ''
      map $http_upgrade $connection_upgrade {
        default upgrade;
        `` close;
      }

      upstream api {
        server localhost:5000;
      }
      upstream web {
        server localhost:8080;
      }

      server {
        listen 8000;
        server_name _;

        location /api {
          proxy_pass http://api;
        }

        location / {
          proxy_pass http://web;
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection $connection_upgrade;
          proxy_set_header Host $host;
        }
      }
    '';
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
