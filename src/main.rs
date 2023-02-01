use dotenvy::dotenv;

use arga_backend::SolrClient;
use arga_backend::http;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    let solr_host = std::env::var("SOLR_URL").expect("No solr URL specified");
    let solr = SolrClient::new(&solr_host);

    let config = http::Config {
        frontend_host,
    };

    http::serve(config, solr).await.expect("Failed to start server");
}
