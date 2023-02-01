use std::net::SocketAddr;
use anyhow::Context as ErrorContext;

use axum::Router;
use axum::http::{Method, HeaderValue};

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::solr_client::SolrClient;

pub mod error;
pub mod search;

use error::Error;


#[derive(Clone)]
pub struct Config {
    /// The host URL path serving the frontend code. This is used
    /// in the CORS layer to allow cross site requests from specific
    /// origins
    pub frontend_host: String,
}


#[derive(Clone)]
pub(crate) struct Context {
    pub config: Config,
    pub solr: SolrClient,
}


pub async fn serve(config: Config, solr: SolrClient) -> anyhow::Result<()> {
    let context = Context {
        config,
        solr,
    };

    let app = router(context)?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}


fn router(context: Context) -> Result<Router, Error> {
    let host = context.config.frontend_host.clone();
    let origin = host.parse::<HeaderValue>().map_err(|_| {
        Error::Configuration(String::from("frontend_host"), host)
    })?;

    let router = Router::new()
        .merge(search::router())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([Method::GET]),
        )
        .with_state(context);

    Ok(router)
}
