use arga_backend::database::Database;
use arga_backend::http;
use diesel::connection::set_default_instrumentation;
use dotenvy::dotenv;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Registry};


#[tokio::main]
async fn main() {
    dotenv().ok();
    serve().await;
}


fn start_tracing() {
    let subscriber = Registry::default();

    println!("Starting debug trace logger for stdout");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info,arga_backend=debug"));

    subscriber
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}


/// Start the HTTP server
async fn serve() {
    start_tracing();

    // realistically will either be 0.0.0.0 or 127.0.0.1 depending on where it will run
    let bind_address = std::env::var("BIND_ADDRESS").expect("No binding address specified");
    let bind_address = bind_address.parse().expect("Failed to parse the binding address");

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    // path to the admin frontend
    let admin_proxy = std::env::var("ADMIN_PROXY").ok();
    let admin_proxy = admin_proxy.map(|proxy| proxy.parse::<axum::http::Uri>().expect("Invalid admin proxy"));

    // show database logging if enabled
    if std::env::var("LOG_DATABASE").is_ok() {
        set_default_instrumentation(Database::simple_logger).expect("Failed to setup database instrumentation");
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
    };

    http::serve(config, database).await.expect("Failed to start server");
}
