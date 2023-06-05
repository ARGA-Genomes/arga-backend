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


/// Start the HTTP server
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
