use dotenvy::dotenv;

use arga_backend::SolrClient;
use arga_backend::http;

use arga_backend::index::providers::{Solr, SolrClient as Client};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let bind_address = std::env::var("BIND_ADDRESS").expect("No binding address specified");
    let bind_address = bind_address.parse().expect("Failed to parse the binding address");

    // used for cors
    let frontend_host = std::env::var("FRONTEND_URL").expect("No frontend URL specified");

    let solr_host = std::env::var("SOLR_URL").expect("No solr URL specified");
    let solr = SolrClient::new(&solr_host);

    let client = Client::new(&solr_host);
    let provider = Solr::new(client);

    let config = http::Config {
        bind_address,
        frontend_host,
    };

    http::serve(config, solr, provider).await.expect("Failed to start server");
}
