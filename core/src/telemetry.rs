//! Shared telemetry configuration for ARGA services
//!
//! This module provides a common interface for setting up OpenTelemetry tracing
//! across different ARGA services (server, tasks, etc.).

#[cfg(feature = "telemetry")]
use opentelemetry::global;
#[cfg(feature = "telemetry")]
use opentelemetry::trace::TracerProvider;
#[cfg(feature = "telemetry")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "telemetry")]
use opentelemetry_sdk::Resource;
#[cfg(feature = "telemetry")]
use tracing_opentelemetry::OpenTelemetryLayer;
#[cfg(feature = "telemetry")]
use tracing_subscriber::EnvFilter;
#[cfg(feature = "telemetry")]
use tracing_subscriber::prelude::*;

/// Configuration for telemetry setup
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,
    /// OTLP endpoint URL
    pub otlp_endpoint: String,
    /// Service name for telemetry
    pub service_name: String,
    /// Service version for telemetry
    pub service_version: String,
    /// Environment filter for logs
    pub log_filter: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("ENABLE_TELEMETRY")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),
            service_name: std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "arga-service".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            log_filter: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }
}

impl TelemetryConfig {
    /// Create a new telemetry configuration with the given service name
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            ..Default::default()
        }
    }

    /// Set the service version, typically from env!("CARGO_PKG_VERSION")
    pub fn with_service_version(mut self, version: &str) -> Self {
        self.service_version = version.to_string();
        self
    }

    /// Set a custom log filter
    pub fn with_log_filter(mut self, filter: &str) -> Self {
        self.log_filter = filter.to_string();
        self
    }

    /// Set a custom OTLP endpoint
    pub fn with_otlp_endpoint(mut self, endpoint: &str) -> Self {
        self.otlp_endpoint = endpoint.to_string();
        self
    }
}

/// Initialize telemetry with the given configuration
pub async fn init_telemetry(config: TelemetryConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "telemetry")]
    {
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_filter));

        if config.enabled {
            println!("Starting telemetry with OpenTelemetry");
            println!("Service: {}", config.service_name);
            println!("Version: {}", config.service_version);
            println!("OTLP Endpoint: {}", config.otlp_endpoint);

            // Create a resource with service information
            let resource = Resource::builder_empty()
                .with_attributes([
                    opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
                    opentelemetry::KeyValue::new("service.version", config.service_version.clone()),
                ])
                .build();

            // Create OTLP exporter
            let exporter_result = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(config.otlp_endpoint)
                .build();

            match exporter_result {
                Ok(exporter) => {
                    // Create tracer provider with batch processing
                    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                        .with_batch_exporter(exporter)
                        .with_resource(resource)
                        .build();

                    // Get the tracer from the provider before setting it global
                    let tracer = tracer_provider.tracer(config.service_name.clone());

                    // Set as global tracer provider
                    global::set_tracer_provider(tracer_provider);

                    // Initialize tracing subscriber with OpenTelemetry layer
                    tracing_subscriber::registry()
                        .with(env_filter)
                        .with(tracing_subscriber::fmt::layer().pretty())
                        .with(OpenTelemetryLayer::new(tracer))
                        .init();

                    println!("OpenTelemetry tracing initialized successfully");
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

        Ok(())
    }

    #[cfg(not(feature = "telemetry"))]
    {
        // Fallback when telemetry feature is not enabled
        use tracing_subscriber::{EnvFilter, fmt};

        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_filter));

        println!("Starting debug trace logger (telemetry feature disabled)");
        fmt().with_env_filter(env_filter).pretty().init();

        Ok(())
    }
}

/// Simple initialization with service name and default configuration
pub async fn init_telemetry_simple(service_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = TelemetryConfig::new(service_name).with_service_version(env!("CARGO_PKG_VERSION"));
    init_telemetry(config).await
}
