use std::collections::HashMap;

use axum::extract::{State, Query};
use axum::{Json, Router};
use axum::routing::{get, post};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::schema;

use crate::http::Context;
use crate::http::error::InternalError;
use crate::index::providers::db::Database;
use crate::index::providers::db::models::{Media, Name, TaxonPhoto};


#[derive(Serialize, Debug)]
struct MediaList {
    total: usize,
    records: Vec<Media>,
}

async fn media(
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<MediaList>, InternalError>
{
    use schema::media::dsl::*;
    use schema::media_observations::dsl as observations;
    let mut conn = db_provider.pool.get().await?;

    let name = params.get("scientific_name").expect("must provide a scientific name parameter");

    // pagination
    let page = parse_int_param(&params, "page", 1);
    let page_size = parse_int_param(&params, "page_size", 5);
    let offset = (page - 1) * page_size;

    let records = media
        .inner_join(observations::media_observations.on(media_id.eq(observations::media_id)))
        .select(media::all_columns())
        .filter(observations::scientific_name.eq(name))
        .order(media_id.desc())
        .offset(offset)
        .limit(page_size)
        .load::<Media>(&mut conn).await?;

    let total: i64 = media
        .inner_join(observations::media_observations.on(media_id.eq(observations::media_id)))
        .filter(observations::scientific_name.eq(name))
        .count()
        .get_result(&mut conn).await?;

    Ok(Json(MediaList {
        total: total as usize,
        records,
    }))
}


async fn main_media(
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<TaxonPhoto>, InternalError>
{
    let mut conn = db_provider.pool.get().await?;

    let name = params.get("scientific_name").expect("must provide a scientific name parameter");

    use schema::{names, taxon_photos};
    let photo = taxon_photos::table
        .select(taxon_photos::all_columns)
        .inner_join(names::table)
        .filter(names::scientific_name.eq(name))
        .get_result::<TaxonPhoto>(&mut conn)
        .await?;

    Ok(Json(photo))
}


#[derive(Deserialize, Debug)]
struct SetMainMedia {
    url: String,
    scientific_name: String,
    source: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
}

async fn upsert_main_media(
    State(db_provider): State<Database>,
    Json(form): Json<SetMainMedia>,
) -> Result<(), InternalError>
{
    // link the main photo as an attribute against the taxa
    use schema::{names, taxon_photos};
    let mut conn = db_provider.pool.get().await?;

    debug!(?form, "setting main image");

    let name: Name = names::table
        .filter(names::scientific_name.eq(form.scientific_name))
        .get_result(&mut conn)
        .await?;

    // delete any previous main images
    diesel::delete(taxon_photos::table)
        .filter(taxon_photos::name_id.eq(name.id))
        .execute(&mut conn)
        .await?;

    // add a taxa photo entry linked to the name
    let photo = TaxonPhoto {
        id: Uuid::new_v4(),
        name_id: name.id,
        url: form.url,
        source: form.source,
        publisher: form.publisher,
        license: form.license,
        rights_holder: form.rights_holder,
    };

    diesel::insert_into(taxon_photos::table)
        .values(&photo)
        .execute(&mut conn)
        .await?;

    debug!(?photo, ?name, "main image set");

    Ok(())
}


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/media", get(media))
        .route("/api/admin/media/main", get(main_media))
        .route("/api/admin/media/main", post(upsert_main_media))
}


fn parse_int_param(params: &HashMap<String, String>, name: &str, default: i64) -> i64 {
    let val = params.get(name).map(|val| val.parse::<i64>().unwrap_or(default)).unwrap_or(default);
    if val <= 0 { 1 } else { val }
}
