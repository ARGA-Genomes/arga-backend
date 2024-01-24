use axum::extract::State;
use axum::{Json, Router};
use axum::routing::{get, post, put, delete};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use crate::http::Context;
use crate::http::error::InternalError;
use crate::database::{schema, Database};
use crate::database::models::Source;


#[derive(Deserialize, Debug)]
struct NewSource {
    pub name: String,
    pub author: String,
    pub rights_holder: String,
    pub access_rights: String,
    pub license: String,
}

#[derive(Deserialize, Debug)]
struct UpdateSource {
    pub id: Uuid,
    pub name: String,
    pub author: String,
    pub rights_holder: String,
    pub access_rights: String,
    pub license: String,
}


async fn sources(State(database): State<Database>) -> Result<Json<Vec<Source>>, InternalError> {
    use schema::sources::dsl::*;
    let mut conn = database.pool.get().await?;

    let records = sources
        .order_by(name)
        .load::<Source>(&mut conn)
        .await?;

    Ok(Json(records))
}

async fn create_sources(
    State(database): State<Database>,
    Json(form): Json<Vec<NewSource>>,
) -> Result<Json<Vec<Source>>, InternalError>
{
    use schema::sources::dsl::*;

    let mut conn = database.pool.get().await?;

    let mut records = Vec::new();
    for row in form {
        records.push(Source {
            id: Uuid::new_v4(),
            name: row.name,
            author: row.author,
            rights_holder: row.rights_holder,
            access_rights: row.access_rights,
            license: row.license,
        })
    }

    let inserted = diesel::insert_into(sources)
        .values(records)
        .get_results(&mut conn)
        .await?;

    Ok(Json(inserted))
}


async fn update_sources(
    State(database): State<Database>,
    Json(form): Json<Vec<UpdateSource>>,
) -> Result<Json<Vec<Source>>, InternalError>
{
    use schema::sources::dsl::*;

    let mut conn = database.pool.get().await?;

    for row in form {
        diesel::update(sources.filter(id.eq(row.id)))
            .set((
                name.eq(row.name),
                author.eq(row.author),
                rights_holder.eq(row.rights_holder),
                access_rights.eq(row.access_rights),
                license.eq(row.license),
            ))
            .execute(&mut conn)
            .await?;
    }

    Ok(Json(vec![]))
}

async fn delete_sources(
    State(database): State<Database>,
    Json(form): Json<Vec<Uuid>>,
) -> Result<Json<Vec<Source>>, InternalError>
{
    use schema::sources::dsl::*;
    let mut conn = database.pool.get().await?;
    diesel::delete(sources.filter(id.eq_any(form))).execute(&mut conn).await?;
    Ok(Json(vec![]))
}


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/sources", get(sources))
        .route("/api/admin/sources", post(create_sources))
        .route("/api/admin/sources", put(update_sources))
        .route("/api/admin/sources", delete(delete_sources))
}
