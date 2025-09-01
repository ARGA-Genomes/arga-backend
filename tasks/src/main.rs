use clap::Parser;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing::info;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

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
    // Get telemetry configuration from environment
    let use_telemetry = std::env::var("ENABLE_TELEMETRY")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info,arga_tasks=debug"));

    if use_telemetry {
        println!("Starting telemetry with OpenTelemetry");

        // Configure OpenTelemetry
        let otlp_endpoint =
            std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string());

        let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "arga-tasks".to_string());

        // Create a resource with service information
        let resource = Resource::builder_empty()
            .with_attributes([
                opentelemetry::KeyValue::new("service.name", service_name),
                opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ])
            .build();

        // Create OTLP exporter
        let exporter_result = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build();

        match exporter_result {
            Ok(exporter) => {
                // Create tracer provider with batch processing
                let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(resource)
                    .build();

                // Get the tracer from the provider before setting it global
                let tracer = tracer_provider.tracer("arga-tasks");

                // Set as global tracer provider
                global::set_tracer_provider(tracer_provider);

                // Initialize tracing subscriber with OpenTelemetry layer
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer().pretty())
                    .with(OpenTelemetryLayer::new(tracer))
                    .init();
            }
            Err(e) => {
                eprintln!("Failed to create OpenTelemetry exporter: {}", e);
                println!("Falling back to debug trace logger for stdout");
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer().pretty())
                    .init();
            }
        }
    }
    else {
        println!("Starting debug trace logger for stdout");
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}
