use tracing_subscriber::{ Registry, EnvFilter };
use tracing_subscriber::prelude::*;

use dotenvy::dotenv;

use arga_backend::http;
use arga_backend::telemetry;

use arga_backend::database::Database;


#[tokio::main]
async fn main() {
    dotenv().ok();
    serve().await;
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
    let db_host = arga_backend::database::get_database_url();
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    let config = http::Config {
        bind_address,
        frontend_host,
    };

    http::serve(config, database).await.expect("Failed to start server");

    telemetry::shutdown();
}
