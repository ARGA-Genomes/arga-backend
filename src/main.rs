use dotenvy::dotenv;

use arga_backend::SolrClient;
use arga_backend::http;

use arga_backend::index::providers::db::Database;
use arga_backend::index::providers::{Solr, SolrClient as Client};
use sqlx::postgres::PgPoolOptions;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    let solr_host = std::env::var("SOLR_URL").expect("No solr URL specified");
    let solr = SolrClient::new(&solr_host);

    let client = Client::new(&solr_host);
    let provider = Solr::new(client);

    let db_host = std::env::var("DATABASE_URL").expect("No database url specified");
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_host).await.expect("can't connect to database");
    let database = Database::new(pool);

    let config = http::Config {
        frontend_host,
    };

    http::serve(config, solr, provider, database).await.expect("Failed to start server");
}
