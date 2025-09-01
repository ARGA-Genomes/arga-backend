use axum::{Json, Router};
use axum::routing::get;
use serde::Serialize;
use tracing::instrument;

use crate::http::Context;
use super::error::Error;


#[derive(Debug, Serialize)]
struct Health {
    healthy: bool,
}

pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/health", get(health))
}


#[instrument]
async fn health() -> Result<Json<Health>, Error> {
    let health = Health {
        healthy: true,
    };

    Ok(Json(health))
}
