use arga_core::models::AccessRightsStatus;
use arga_core::models::DataReuseStatus;
use arga_core::models::Dataset;
use arga_core::models::SourceContentType;
use axum::extract::State;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use crate::database::{schema, Database};
use crate::http::error::InternalError;
use crate::http::Context;

#[derive(Deserialize, Debug)]
struct NewDataset {
    pub source_id: Uuid,
    pub global_id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub citation: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub reuse_pill: Option<DataReuseStatus>,
    pub access_pill: Option<AccessRightsStatus>,
    pub publication_year: Option<i16>,
    pub content_type: Option<SourceContentType>,
}

#[derive(Deserialize, Debug)]
struct UpdateDataset {
    pub id: Uuid,
    pub source_id: Uuid,
    pub global_id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub citation: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub reuse_pill: Option<DataReuseStatus>,
    pub access_pill: Option<AccessRightsStatus>,
    pub publication_year: Option<i16>,
    pub content_type: Option<SourceContentType>,
}

async fn datasets(State(database): State<Database>) -> Result<Json<Vec<Dataset>>, InternalError> {
    use schema::datasets::dsl::*;
    let mut conn = database.pool.get().await?;

    let records = datasets.order_by(global_id).load::<Dataset>(&mut conn).await?;

    Ok(Json(records))
}

async fn create_datasets(
    State(database): State<Database>,
    Json(form): Json<Vec<NewDataset>>,
) -> Result<Json<Vec<Dataset>>, InternalError> {
    use schema::datasets::dsl::*;

    let mut conn = database.pool.get().await?;

    let mut records = Vec::new();
    for row in form {
        records.push(Dataset {
            id: Uuid::new_v4(),
            source_id: row.source_id,
            global_id: row.global_id,
            name: row.name,
            short_name: row.short_name,
            description: row.description,
            url: row.url,
            citation: row.citation,
            license: row.license,
            rights_holder: row.rights_holder,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            reuse_pill: row.reuse_pill,
            access_pill: row.access_pill,
            publication_year: row.publication_year,
            content_type: row.content_type,
        })
    }

    let inserted = diesel::insert_into(datasets)
        .values(records)
        .get_results(&mut conn)
        .await?;

    Ok(Json(inserted))
}

async fn update_datasets(
    State(database): State<Database>,
    Json(form): Json<Vec<UpdateDataset>>,
) -> Result<Json<Vec<Dataset>>, InternalError> {
    use schema::datasets::dsl::*;

    let mut conn = database.pool.get().await?;

    for row in form {
        diesel::update(datasets.filter(id.eq(row.id)))
            .set((
                source_id.eq(row.source_id),
                global_id.eq(row.global_id),
                name.eq(row.name),
                short_name.eq(row.short_name),
                description.eq(row.description),
                url.eq(row.url),
                citation.eq(row.citation),
                license.eq(row.license),
                rights_holder.eq(row.rights_holder),
                updated_at.eq(chrono::Utc::now()),
                reuse_pill.eq(row.reuse_pill),
                access_pill.eq(row.access_pill),
                publication_year.eq(row.publication_year),
                content_type.eq(row.content_type),
            ))
            .execute(&mut conn)
            .await?;
    }

    Ok(Json(vec![]))
}

async fn delete_datasets(
    State(database): State<Database>,
    Json(form): Json<Vec<Uuid>>,
) -> Result<Json<Vec<Dataset>>, InternalError> {
    use schema::datasets::dsl::*;
    let mut conn = database.pool.get().await?;
    diesel::delete(datasets.filter(id.eq_any(form)))
        .execute(&mut conn)
        .await?;
    Ok(Json(vec![]))
}

/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/datasets", get(datasets))
        .route("/api/admin/datasets", post(create_datasets))
        .route("/api/admin/datasets", put(update_datasets))
        .route("/api/admin/datasets", delete(delete_datasets))
}
