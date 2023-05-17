use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use axum_login::secrecy::{SecretString, ExposeSecret};
use tracing_subscriber::{ Registry, EnvFilter };
use tracing_subscriber::prelude::*;

use dotenvy::dotenv;
use clap::Parser;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use arga_backend::http;
use arga_backend::telemetry;
use arga_backend::schema;
use arga_backend::workers;
use arga_backend::search;

use arga_backend::index::providers::{Solr, SolrClient};
use arga_backend::index::providers::db::Database;


/// The ARGA backend
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
   #[command(subcommand)]
   command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run and manage worker processes
    #[command(subcommand)]
    Workers(workers::Command),

    /// Run and manage the search index
    #[command(subcommand)]
    Search(search::Command),

    /// Create a new admin user
    CreateAdmin {
        /// The full name of the new admin user
        name: String,
        /// The email address of the new admin user
        email: String,
        /// A generated and safe password for the new admin user
        password: String,
    },
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::CreateAdmin { name, email, password }) => {
            let secret = SecretString::new(password.to_string());
            create_admin(name, email, secret).await;
        }
        Some(Commands::Workers(command)) => workers::process_command(command),
        Some(Commands::Search(command)) => search::process_command(command).await,
        None => serve().await,
    }
}

async fn create_admin(name: &str, email: &str, password: SecretString) {
    use schema::users::dsl as dsl;

    let db_host = std::env::var("DATABASE_URL").expect("No database url specified");
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");
    let mut pool = database.pool.get().await.unwrap();

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(&password.expose_secret().as_bytes(), &salt).unwrap().to_string();

    diesel::insert_into(dsl::users)
        .values((
            dsl::name.eq(name),
            dsl::email.eq(email),
            dsl::password_hash.eq(hash),
            dsl::password_salt.eq(salt.to_string()),
            dsl::user_role.eq("admin"),
        ))
        .execute(&mut pool)
        .await.unwrap();
}


async fn serve() {
    // setup tracing with opentelemetry support. this allows us to use tracing macros
    // for both logging and metrics
    let subscriber = Registry::default();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info,arga_backend=trace"));

    let controller = telemetry::init_metrics().unwrap();
    let metrics =  tracing_opentelemetry::MetricsLayer::new(controller);

    let tracer = telemetry::init_tracer().unwrap();
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    subscriber
        .with(env_filter)
        .with(opentelemetry)
        .with(metrics)
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    // realistically will either be 0.0.0.0 or 127.0.0.1 depending on where it will run
    let bind_address = std::env::var("BIND_ADDRESS").expect("No binding address specified");
    let bind_address = bind_address.parse().expect("Failed to parse the binding address");

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    // because the entire backend is in a crate we instantiate providers and any other services
    // here so that it is explicitly defined
    let solr_host = std::env::var("SOLR_URL").expect("No solr URL specified");
    let client = SolrClient::new(&solr_host);
    let provider = Solr::new(client);

    let db_host = std::env::var("DATABASE_URL").expect("No database url specified");
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    let config = http::Config {
        bind_address,
        frontend_host,
    };

    http::serve(config, provider, database).await.expect("Failed to start server");

    telemetry::shutdown();
}
