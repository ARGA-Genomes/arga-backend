use tracing_subscriber::{ Registry, EnvFilter };
use tracing_subscriber::prelude::*;

use dotenvy::dotenv;
use clap::Parser;

use arga_backend::http;
use arga_backend::telemetry;
use arga_backend::workers;
use arga_backend::search;

use arga_backend::index::providers::{Solr, SolrClient};
use arga_backend::database::Database;

mod tasks;


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
        Some(Commands::CreateAdmin { name, email, password }) => tasks::admin::create_admin(name, email, password).await,
        Some(Commands::Workers(command)) => workers::process_command(command),
        Some(Commands::Search(command)) => search::process_command(command).await,
        None => serve().await,
    }
}


fn start_tracing() {
    // setup tracing with opentelemetry support. this allows us to use tracing macros
    // for both logging and metrics
    let subscriber = Registry::default();

    if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        println!("Starting trace logger with telemetry collector at: {}", endpoint);

        let controller = telemetry::init_metrics().expect("Failed to initialise telemetry metrics");
        let metrics =  tracing_opentelemetry::MetricsLayer::new(controller);

        let tracer = telemetry::init_tracer().expect("Failed to initialise telemetry tracer");
        let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

        subscriber
            .with(env_filter)
            .with(opentelemetry)
            .with(metrics)
            .init();
    }
    else {
        println!("Starting debug trace logger for stdout");

        let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info,arga_backend=debug"));

        subscriber
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}


/// Start the HTTP server
async fn serve() {
    start_tracing();

    // realistically will either be 0.0.0.0 or 127.0.0.1 depending on where it will run
    let bind_address = std::env::var("BIND_ADDRESS").expect("No binding address specified");
    let bind_address = bind_address.parse().expect("Failed to parse the binding address");

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    // because the entire backend is in a crate we instantiate providers and any other services
    // here so that it is explicitly defined
    let solr_host = std::env::var("SOLR_URL").expect("No solr URL specified");
    let client = SolrClient::new(&solr_host);
    let solr = Solr::new(client);

    let db_host = get_database_url();
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    let config = http::Config {
        bind_address,
        frontend_host,
    };

    http::serve(config, database, solr).await.expect("Failed to start server");

    telemetry::shutdown();
}


fn get_database_url() -> String {
    match std::env::var("DATABASE_URL") {
        Ok(url) => url.to_string(),
        Err(_) => {
            tracing::info!("DATABASE_URL not specified. Building URL from other env vars");
            let host = std::env::var("DATABASE_HOST").expect("Must specify a database host");
            let port = std::env::var("DATABASE_PORT").expect("Must specify a database port");
            let user = std::env::var("DATABASE_USER").expect("Must specify a database user");
            let pass = std::env::var("DATABASE_PASS").expect("Must specify a database pass");
            let name = std::env::var("DATABASE_NAME").expect("Must specify a database name");

            format!("postgresql://{user}:{pass}@{host}:{port}/{name}")
        }
    }
}
