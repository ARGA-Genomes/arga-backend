use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context as ErrorContext;
use arga_core::search::SearchIndex;
use axum::Router;
use axum::extract::FromRef;
use axum::http::{HeaderValue, Method, header};
use tower_http::CompressionLevel;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::DecompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::sensitive_headers::{SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::database::Database;

pub mod admin;
pub mod auth;
pub mod error;
pub mod graphql;
pub mod health;

pub use error::Error;


#[derive(Clone, Debug)]
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


/// The state made avaialbe to every request.
#[derive(Clone)]
pub(crate) struct Context {
    pub config: Config,
    pub database: Database,
    pub search: SearchIndex,
}

impl FromRef<Context> for Database {
    fn from_ref(state: &Context) -> Self {
        state.database.clone()
    }
}

/// Create the context and serve the API.
///
/// This will create the context based on the configuration
/// and kick off the http server.
pub async fn serve(config: Config, database: Database) -> anyhow::Result<()> {
    let addr = config.bind_address.clone();

    let context = Context {
        config,
        database,
        search: SearchIndex::open()?,
    };

    let app = router(context)?;

    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.context("error running HTTP server")
}

/// The root router.
///
/// Sets up the middleware and merges the REST and GraphQL API
/// into the same namespace. The context present in every request
/// is also moved here and cloned out to any sub-routers.
fn router(context: Context) -> Result<Router, Error> {
    let host = context.config.frontend_host.clone();
    let origin = host
        .parse::<HeaderValue>()
        .map_err(|_| Error::Configuration(String::from("frontend_host"), host))?;

    // headers we want to treat as sensitive. sensitive heaers helps us not accidentally leak
    // secrets in logs or apply compression to payloads that undermine the security of the request
    let headers: Arc<[_]> = Arc::new([
        header::AUTHORIZATION,
        header::PROXY_AUTHORIZATION,
        header::COOKIE,
        header::SET_COOKIE,
    ]);

    let service = tower::ServiceBuilder::new()
        // mark sensitive headers. do it at the start for request headers to avoid
        // leaking it to middlewares
        .layer(SetSensitiveRequestHeadersLayer::from_shared(Arc::clone(&headers)))
        // add tracing support for all requests coming through
        .layer(TraceLayer::new_for_http())
        // limit the payload size of the request. it does this by looking at the Content-Length
        // header but hyper will also bail at this limit if the payload is actually bigger
        // that what the header declares
        .layer(RequestBodyLimitLayer::new(4096))
        // add request compression support for gzip and brotli compression
        .layer(
            CompressionLayer::new()
                .gzip(true)
                .br(true)
                .quality(CompressionLevel::Best),
        )
        // allow compressed requests to be made with gzip and brotli
        .layer(DecompressionLayer::new().gzip(true).br(true))
        // hard timeout for requests that take to long to complete
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
        // mark sensitive headers. do it at the end for response headers to avoid
        // leaking it to middlewares
        .layer(SetSensitiveResponseHeadersLayer::from_shared(headers))
        // cross origin resource sharing. we only want the public API to be accessible
        // to anyone, for admin and everything else it should be limited to our servers only.
        // for now we just limit it to our own servers
        .layer(
            CorsLayer::permissive(), // CorsLayer::new()
                                     //     .allow_methods([Method::GET, Method::OPTIONS])
                                     //     .allow_origin(origin),
        );


    let router = Router::new()
        .merge(health::router())
        .merge(graphql::router(context.clone()))
        .nest("/api/admin", admin::router(context.clone()))
        .layer(service)
        .with_state(context);

    Ok(router)
}
