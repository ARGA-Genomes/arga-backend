use anyhow::Context as ErrorContext;
use std::net::SocketAddr;

use axum::http::{HeaderValue, Method};
use axum::Router;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::FeatureClient;
use crate::features::Features;
use crate::index::providers::ala::Ala;
use crate::index::providers::db::Database;
use crate::index::providers::solr::Solr;

pub mod error;
pub mod graphql;
pub mod health;

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
pub struct Providers {
    pub solr: Solr,
    pub ala: Ala,
    pub db: Database,
}


/// The state made avaialbe to every request.
#[derive(Clone)]
pub(crate) struct Context {
    pub config: Config,
    pub provider: Solr,
    pub ala_provider: Ala,
    pub db_provider: Database,
    pub features: FeatureClient,
}

/// Create the context and serve the API.
///
/// This will create the context based on the configuration
/// and kick off the http server.
pub async fn serve(config: Config, provider: Solr, db_provider: Database) -> anyhow::Result<()> {
    let addr = config.bind_address.clone();

    let context = Context {
        config,
        provider,
        db_provider,
        ala_provider: Ala::new(),
        features: FeatureClient::new(),
    };

    let app = router(context)?;

    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}

/// The root router.
///
/// Sets up the middleware and merges the REST and GraphQL API
/// into the same namespace. The context present in every request
/// is also moved here and cloned out to any sub-routers.
fn router(context: Context) -> Result<Router, Error> {
    let with_tracing = context.features.is_enabled(Features::OpenTelemetry);

    let host = context.config.frontend_host.clone();
    let origin = host
        .parse::<HeaderValue>()
        .map_err(|_| Error::Configuration(String::from("frontend_host"), host))?;

    let mut router = Router::new()
        .merge(health::router())
        .merge(graphql::router(context.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([Method::GET]),
        )
        .with_state(context);

    if let Ok(true) = with_tracing {
        tracing::info!("Enabling axum tracing layer");
        router = router.layer(TraceLayer::new_for_http());
    }

    Ok(router)
}
