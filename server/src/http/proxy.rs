use axum::Router;
use axum::body::Body;
use axum::extract::{NestedPath, Request, State};
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::http::uri::Uri;
use axum::response::{IntoResponse, Response};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use tower_http::services::ServeDir;
use tracing::{debug, info, warn};

use crate::http::Context;

// client to enable frontend reverse proxying
pub(crate) type ProxyClient = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[derive(Clone)]
pub(crate) enum Proxy {
    Static(Uri),
    Http { uri: Uri, client: ProxyClient },
}

pub(crate) fn build_proxy(uri: Uri) -> Proxy {
    match uri.scheme_str() {
        Some("http") => Proxy::Http {
            uri,
            client: hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
                .build(HttpConnector::new()),
        },
        _ => Proxy::Static(uri),
    }
}

// tunnel the request to teh underlying endpoint. should only be used for development
async fn frontend_proxy(
    State((uri, client)): State<(Uri, ProxyClient)>,
    root: NestedPath,
    mut req: Request,
) -> Result<Response, StatusCode> {
    let root = root.as_str();
    let mut parts = uri.into_parts();
    parts.path_and_query = req
        .uri()
        .path_and_query()
        .map(|path| format!("{root}{path}").parse().expect("Invalid path and query"));

    let uri = Uri::from_parts(parts).expect("Invalid URI");
    debug!(%uri, "Proxying");
    *req.uri_mut() = uri;

    Ok(client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_response())
}


pub(crate) fn admin_web_router(context: Context) -> Router<Context> {
    match context.proxy {
        Some(Proxy::Http { uri, client }) => {
            info!(%uri, "Serving admin frontend via reverse proxy");
            let handler = frontend_proxy.with_state((uri, client));
            Router::new()
                .route_service("/", handler.clone())
                .fallback_service(handler)
        }
        Some(Proxy::Static(uri)) => {
            info!(%uri, "Serving admin frontend via static files");
            let serve_dir = ServeDir::new(uri.path().trim_start_matches("/"));
            Router::new().fallback_service(serve_dir)
        }
        None => {
            warn!("No admin frontend proxy specified. Admin won't be accessible");
            Router::new()
        }
    }
}
