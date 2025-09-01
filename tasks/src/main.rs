#[cfg(feature = "telemetry")]
use arga_core::telemetry::{self, TelemetryConfig};
use clap::Parser;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod admin;
pub mod dataset;
// pub mod search;
pub mod data;
pub mod reports;


/// The ARGA backend
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run and manage the search index
    // #[command(subcommand)]
    // Search(search::Command),

    /// Create a new admin user
    CreateAdmin {
        /// The full name of the new admin user
        name: String,
        /// The email address of the new admin user
        email: String,
        /// A generated and safe password for the new admin user
        password: String,
    },

    /// Queue an import job for a dataset
    Dataset {
        /// The worker job type that should process the file
        worker: String,
        /// The global ID of the dataset to process the data as
        dataset: String,
        /// The path to the file being imported
        path: String,
        /// A list of dataset global IDs that the import can use when
        /// matching to existing data
        isolation_context: Vec<String>,
    },

    /// Perform tasks on raw data sets
    #[command(subcommand)]
    Data(data::Command),

    /// Create reports related to the database
    #[command(subcommand)]
    Reports(reports::Command),
}


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    start_tracing().await;

    let cli = Cli::parse();
    info!("Starting ARGA tasks");

    match &cli.command {
        Commands::CreateAdmin { name, email, password } => {
            info!("Creating admin user");
            admin::create_admin(name, email, password)
        }
        // Commands::Search(command) => search::process_command(command),
        Commands::Data(command) => {
            info!("Processing data command");
            data::process_command(command)
        }
        Commands::Reports(command) => {
            info!("Processing reports command");
            reports::process_command(command)
        }
        Commands::Dataset {
            worker,
            dataset,
            isolation_context,
            path,
        } => {
            info!(worker = %worker, dataset = %dataset, "Importing dataset");
            dataset::import(worker, dataset, isolation_context, path)
        }
    }
}

async fn start_tracing() {
    // Initialize telemetry using shared module
    #[cfg(feature = "telemetry")]
    {
        let telemetry_config = TelemetryConfig::new("arga-tasks").with_service_version(env!("CARGO_PKG_VERSION"));
        if let Err(e) = telemetry::init_telemetry(telemetry_config).await {
            eprintln!("Failed to initialize telemetry: {}", e);
            // Fall back to basic tracing
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().pretty())
                .with(tracing_subscriber::filter::EnvFilter::from_default_env())
                .init();
        }
    }

    #[cfg(not(feature = "telemetry"))]
    {
        // Initialize basic tracing subscriber when telemetry is disabled
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().pretty())
            .with(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();
    }
}
