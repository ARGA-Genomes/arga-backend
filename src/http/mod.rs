use anyhow::Context as ErrorContext;
use std::net::SocketAddr;

use axum::http::{HeaderValue, Method};
use axum::Router;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::index::providers::solr::Solr;
use crate::solr_client::SolrClient;

pub mod error;
pub mod graphql;
pub mod search;

use error::Error;

#[derive(Clone)]
pub struct Config {
    /// The address to bind the http listener to. For local development
    /// this will almost always be 127.0.0.1:5000. For production it needs
    /// to bind to a public interface which should be something like 0.0.0.0:5000
    pub bind_address: SocketAddr,

    /// The host URL path serving the frontend code. This is used
    /// in the CORS layer to allow cross site requests from specific
    /// origins
    pub frontend_host: String,
}

#[derive(Clone)]
pub(crate) struct Context {
    pub config: Config,
    pub solr: SolrClient,
    pub provider: Solr,
}

pub async fn serve(config: Config, solr: SolrClient, provider: Solr) -> anyhow::Result<()> {
    let addr = config.bind_address.clone();

    let context = Context {
        config,
        solr,
        provider,
    };

    let app = router(context)?;

    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}

fn router(context: Context) -> Result<Router, Error> {
    let host = context.config.frontend_host.clone();
    let origin = host
        .parse::<HeaderValue>()
        .map_err(|_| Error::Configuration(String::from("frontend_host"), host))?;

    let router = Router::new()
        .merge(search::router())
        .merge(graphql::router(context.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([Method::GET]),
        )
        .with_state(context);

    Ok(router)
}
