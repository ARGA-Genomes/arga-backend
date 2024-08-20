The arga-core crate provides core functionality to all tools and servers that need database, search index, or CRDT integration. This is effectively the synchronisation point between ARGA projects and the backend web service.

## Consumers
The applications and tools that leverage the core crate
- backend-server
- backend-workers
- search-indexer
- oplogger

## Migrations

The database schema is defined and maintained in this crate with `diesel-cli`. This can be installed with `cargo install diesel-cli` if you already have a rust environment.

All migration files are just SQL files called `up.sql` and `down.sql`, with `up` being the migration to apply and `down` being the query that reverts the migration. All migration files are stored in a timestamped folder under `migrations`. The exception being the stabilised migrations that are order by user defined numbers rather than timestamps, eg. `00000000000002_create_datasets`. The order that migrations run is purely lexicographic, so what you see in the editor should be the order the migrations run in.

### Running migrations
To run a migration make sure that the `.env` file in the repository root has an entry for the `DATABASE_URL` pointing to your postgres installation. For example, `DATABASE_URL=postgres://gsterjov@localhost/arga`. You can then run all pending migrations bringing your database up to date by executing

``` sh
diesel migration run
```

you can also revert or redo migrations by respectively executing

``` sh
diesel migration revert
diesel migration redo
```

The diesel-cli is well documented, feel free to append `--help` to any command to find all the functionality it offers.

### Creating migrations
If you want to make changes to the database then use a command like the following

``` sh
diesel migration generate create_datasets_table
diesel migration generate add_columns_to_datasets_table
diesel migration generate remove_unused_columns_from_datasets_table
```
diesel-cli will create the `up.sql` and `down.sql` files within a folder using the text after `generate` prepended with a timestamp. Edit the relevant sql files and write a query that will modify the database the way you need. This is a raw sql query that will ultimately be executed with a migration user on the servers, so many postgres features should be available to you but make sure to test against staging in case the user doesn't have the necessary permissions on the server stack.

The name used for the migration can be anything. Try to make it informative including the action its doing and the tables it affects to aid in debugging.

### Post migration
When you run migrations diesel-cli will automatically regenerate the `src/schema.rs` file. This is a rust file using the diesel DSL to define all tables, types, relations, and types. Its the core file used to power the diesel query builder in the backend and related tools.
A manually maintained file called `src/schema_gnl.rs` sits alongside the schema file. It uses the same DSL to define views and materialized views so they can be used as regular tables in the backend and related tools. We need to manually maintain these as diesel-cli does not have support for generating schema definitions for views.

If you create or modify views be sure to update the `schema_gnl.rs` file to reflect the new definition otherwise we will run into errors with schema mismatches.

Just by running the migration you will get an updated `schema.rs` file which should be committed along with the migration files. If you modify the migration without running it but need the schema file updated then run

``` sh
diesel print-schema
```
to get the same output as a migration run would get.
