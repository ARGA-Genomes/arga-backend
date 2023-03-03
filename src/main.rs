use tracing_subscriber::{ Registry, EnvFilter };
use tracing_subscriber::prelude::*;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

use arga_backend::http;
use arga_backend::telemetry;

use arga_backend::index::providers::{Solr, SolrClient};
use arga_backend::index::providers::db::Database;


#[tokio::main]
async fn main() {
    dotenv().ok();

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


    let bind_address = std::env::var("BIND_ADDRESS").expect("No binding address specified");
    let bind_address = bind_address.parse().expect("Failed to parse the binding address");

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    let solr_host = std::env::var("SOLR_URL").expect("No solr URL specified");
    let client = SolrClient::new(&solr_host);
    let provider = Solr::new(client);

    let db_host = std::env::var("DATABASE_URL").expect("No database url specified");
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_host).await.expect("can't connect to database");
    let database = Database::new(pool);

    let config = http::Config {
        bind_address,
        frontend_host,
    };

    http::serve(config, provider).await.expect("Failed to start server");

    telemetry::shutdown();
}
