use axum::extract::{State, Path};
use axum::{Json, Router};
use axum::routing::{get, delete};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

use crate::http::Context;
use crate::http::error::InternalError;
use crate::database::{schema, Database};
use crate::database::models::NameList;


#[derive(Debug, Serialize)]
struct Lists {
    total: usize,
    records: Vec<NameList>,
}

async fn lists(
    State(db_provider): State<Database>,
) -> Result<Json<Lists>, InternalError>
{
    use schema::name_lists::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let records = name_lists
        .load::<NameList>(&mut conn)
        .await?;

    Ok(Json(Lists {
        total: records.len(),
        records,
    }))
}

async fn show_list(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<NameList>, InternalError>
{
    use schema::name_lists::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = name_lists
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn delete_list(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<NameList>, InternalError>
{
    use schema::name_lists::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::delete(name_lists)
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/lists", get(lists))
        .route("/api/admin/lists/:uuid", get(show_list))
        .route("/api/admin/lists/:uuid", delete(delete_list))
}
