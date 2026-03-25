This repo is an [axum](https://github.com/tokio-rs/axum/) backend server written in Rust and serving a GraphQL API abstracting the ARGA index.

## Getting Started

This repo is built against the latest stable version of rust.

The configuration of the backend server happens via environment variables. First copy the `.env.sample` file:

```bash
cp .env.sample .env
vim .env
```

and then set the variables to point at your Solr and [arga-frontend](https://github.com/ARGA-Genomes/arga-frontend) services. The frontend service is only useful for development as it'll add your nodejs server to the CORS list allowing it to make requests. In production this wont be necessary.

### First-time setup

If this is your first time running the backend server, you'll need to setup the Postgres database and [tantivy](https://github.com/quickwit-oss/tantivy) search index first.

Open a new terminal tab, and run `devenv up` to start postgres on your machine.

The ARGA database can then be created by connecting to your postgres instance via `psql -d postgres` and running:

```postgres
CREATE DATABASE arga;
```

Once created, to follow the instructions in `core/README.md` to setup the database schema. In short, make sure you have `DATABASE_URL` set as an environment variable, and then run the following command from the `core` folder:

```bash
atlas migrate apply --env arga
```

You can also restore from a database dump (see [pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html) and [pg_restore](https://www.postgresql.org/docs/current/app-pgrestore.html)).

Once complete, create a `.index` folder at the root of the project directory, and run:

```bash
cargo run --bin arga-tasks search create
```


### Running the backend server

To run a development server:

```bash
cargo run --bin arga-backend
```

To build a production version and serve it with the nextjs server (optional)

```bash
cargo build --release
./target/release/arga-backend
```


## Reproducible Builds

Included in the repo is a [devenv](https://devenv.sh) configuration to enable a declarative and reproducible environment. This leverages the nix package manager to provide both needed system dependencies as well as convenient developer tools without having to worry about version compatibility.

To get started install `devenv` and then enter the shell by running:

```bash
devenv shell
```

This will take some time to download rust and various tools like rust-analyzer. Once inside the above commands in `Getting Started` should work with the correct dependencies.
If you're messing around with the devenv or nix flake files don't forget to occasionally do a garbage collect by running:

```bash
devenv gc
```

This will remove any dependencies no longer used. Keep in mind that the nix package manager does not use your system packages so it might take up more disk space than you would expect. For example if installing nodejs it will also install the locked dependencies of nodejs as well, and so on.

### direnv

To avoid clobbering other workflows the `.envrc` file created by devenv isn't committed. To automatically enter the shell when you enter the repo directory add the following `.envrc` file to the project root:

```
watch_file devenv.nix
watch_file devenv.yaml
watch_file devenv.lock
eval "$(devenv print-dev-env)"
```
