use arga_backend::database::Database;
use arga_backend::http;
use diesel::connection::set_default_instrumentation;
use dotenvy::dotenv;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;


#[tokio::main]
async fn main() {
    dotenv().ok();
    start_tracing().await;
    serve().await;
}


async fn start_tracing() {
    // Get telemetry configuration from environment
    let use_telemetry = std::env::var("ENABLE_TELEMETRY")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info,arga_backend=debug"));

    if use_telemetry {
        println!("Starting telemetry with OpenTelemetry");

        // Configure OpenTelemetry
        let otlp_endpoint =
            std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string());

        let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "arga-backend".to_string());

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
                let tracer = tracer_provider.tracer("arga-backend");

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


/// Start the HTTP server
async fn serve() {
    // realistically will either be 0.0.0.0 or 127.0.0.1 depending on where it will run
    let bind_address = std::env::var("BIND_ADDRESS").expect("No binding address specified");
    let bind_address = bind_address.parse().expect("Failed to parse the binding address");

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    // path to the admin frontend
    let admin_proxy = std::env::var("ADMIN_PROXY").ok();
    let admin_proxy = admin_proxy.map(|proxy| proxy.parse::<axum::http::Uri>().expect("Invalid admin proxy"));

    // cache configuration
    let cache_url = std::env::var("CACHE_URL").ok();
    let cache_ttl = std::env::var("CACHE_TTL")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()
        .expect("Invalid CACHE_TTL value");

    // cache skip pattern from environment
    let cache_skip_pattern = std::env::var("CACHE_SKIP_PATTERN")
        .ok()
        .filter(|s| !s.trim().is_empty());

    // show database logging if enabled
    if std::env::var("LOG_DATABASE").is_ok() {
        let use_telemetry = std::env::var("ENABLE_TELEMETRY")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        if use_telemetry {
            set_default_instrumentation(Database::detailed_logger).expect("Failed to setup database instrumentation");
        }
        else {
            set_default_instrumentation(Database::simple_logger).expect("Failed to setup database instrumentation");
        }
    }

    // because the entire backend is in a crate we instantiate providers and any other services
    // here so that it is explicitly defined
    let db_host = arga_backend::database::get_database_url();

    let database = Database::connect(&db_host)
        .await
        .expect("Failed to connect to the database");

    let config = http::Config {
        bind_address,
        frontend_host,
        admin_proxy,
        cache_url,
        cache_ttl,
        cache_skip_pattern,
    };

    http::serve(config, database).await.expect("Failed to start server");
}
