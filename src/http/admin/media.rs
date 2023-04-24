use std::collections::HashMap;

use axum::extract::{State, Query, Path};
use axum::{Json, Router};
use axum::routing::{get, post};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;

use crate::schema;
use crate::schema_gnl;

use crate::http::Context;
use crate::http::error::InternalError;
use crate::index::providers::db::Database;
use crate::index::providers::db::models::{Media, ArgaTaxon, Attribute, ObjectValueString, Object, ObjectString};


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


async fn main_media(
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<Media>, InternalError>
{
    let mut conn = db_provider.pool.get().await?;

    let name = params.get("scientific_name").expect("must provide a scientific name parameter");

    use schema_gnl::gnl::dsl as taxa;
    let taxon: ArgaTaxon = taxa::gnl
        .filter(taxa::canonical_name.eq(name))
        .get_result(&mut conn).await?;

    use schema_gnl::eav_strings::dsl as attrs;
    let curated_image = attrs::eav_strings
        .filter(attrs::name.eq("curatedMainImage"))
        .filter(attrs::entity_id.eq(taxon.id))
        .get_result::<ObjectString>(&mut conn).await?;

    let media_uuid = Uuid::parse_str(&curated_image.value)?;

    use schema::media::dsl::*;
    let record: Media = media
        .filter(id.eq(media_uuid))
        .get_result(&mut conn).await?;

    Ok(Json(record))
}


#[derive(Deserialize, Debug)]
struct SetMainMedia {
    species: String,
}

async fn upsert_main_media(
    Path(media_uuid): Path<Uuid>,
    State(db_provider): State<Database>,
    Json(form): Json<SetMainMedia>,
) -> Result<(), InternalError>
{
    // link the main photo as an attribute against the taxa
    use schema_gnl::gnl::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let taxon: ArgaTaxon = gnl
        .filter(canonical_name.eq(form.species))
        .get_result(&mut conn).await?;

    debug!(?media_uuid, ?taxon, "setting main image");

    use schema::attributes::dsl::*;
    let attr: Attribute = attributes
        .filter(name.eq("curatedMainImage"))
        .get_result(&mut conn).await?;

    // check for an existing attribute first
    use schema_gnl::eav_strings::dsl as attrs;
    let record = attrs::eav_strings
        .filter(attrs::attribute_id.eq(attr.id))
        .filter(attrs::entity_id.eq(taxon.id))
        .get_result::<ObjectString>(&mut conn).await;

    match record {
        Ok(rec) => {
            debug!(?rec, "updating");
            use schema::object_values_string::dsl::*;
            diesel::update(object_values_string)
                .filter(id.eq(rec.value_id))
                .set(value.eq(media_uuid.to_string()))
                .execute(&mut conn).await?;
            Ok(())
        }
        Err(diesel::result::Error::NotFound) => {
            let value = ObjectValueString {
                id: Uuid::new_v4(),
                value: media_uuid.to_string(),
            };

            let object = Object {
                id: Uuid::new_v4(),
                entity_id: taxon.id,
                attribute_id: attr.id,
                value_id: value.id,
            };

            debug!(?value, "inserting");
            use schema::object_values_string::dsl::object_values_string;
            diesel::insert_into(object_values_string).values(value).execute(&mut conn).await?;

            use schema::objects::dsl::objects;
            diesel::insert_into(objects).values(object).execute(&mut conn).await?;
            Ok(())
        }
        Err(err) => {
            Err(err.into())
        }
    }
}


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/media", get(media))
        .route("/api/admin/media/main", get(main_media))
        .route("/api/admin/media/:uuid/main", post(upsert_main_media))
}
