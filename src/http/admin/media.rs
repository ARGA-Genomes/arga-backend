use std::collections::HashMap;

use axum::extract::{State, Query};
use axum::{Json, Router};
use axum::routing::get;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::schema;

use crate::http::Context;
use crate::http::error::InternalError;
use crate::index::providers::db::Database;
use crate::index::providers::db::models::Media;


async fn media(
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<Vec<Media>>, InternalError>
{
    use schema::media::dsl::*;
    use schema::media_observations::dsl as observations;
    let mut conn = db_provider.pool.get().await?;

    let name = params.get("scientific_name").expect("must provide a scientific name parameter");

    let records = media
        .inner_join(observations::media_observations.on(media_id.eq(observations::media_id)))
        .select(media::all_columns())
        .filter(observations::scientific_name.eq(name))
        .order(media_id.desc())
        .limit(20)
        .load::<Media>(&mut conn).await?;

    Ok(Json(records))
}

/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/media", get(media))
}
