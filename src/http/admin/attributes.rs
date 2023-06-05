use std::collections::HashMap;

use axum::extract::{State, Query, Path};
use axum::{Json, Router};
use axum::routing::{get, post, put, delete};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::http::Context;
use crate::http::error::InternalError;
use crate::database::{schema, Database};
use crate::database::models::{Attribute, AttributeDataType};


#[derive(Debug, Serialize)]
struct AttributeList {
    total: usize,
    records: Vec<Attribute>,
}

async fn attributes(
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<AttributeList>, InternalError>
{
    use schema::attributes::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    // pagination
    let page = parse_int_param(&params, "page", 1);
    let page_size = parse_int_param(&params, "page_size", 20);
    let offset = (page - 1) * page_size;

    let records = attributes
        .order_by(name)
        .offset(offset)
        .limit(page_size)
        .load::<Attribute>(&mut conn).await?;

    let total: i64 = attributes.count().get_result(&mut conn).await?;

    Ok(Json(AttributeList {
        total: total as usize,
        records,
    }))
}

#[derive(Deserialize, Insertable, AsChangeset, Debug)]
#[diesel(table_name = schema::attributes)]
struct NewAttribute {
    name: String,
    data_type: AttributeDataType,
    description: Option<String>,
    reference_url: Option<String>,
}

async fn create_attribute(
    State(db_provider): State<Database>,
    Json(form): Json<NewAttribute>,
) -> Result<Json<Attribute>, InternalError>
{
    use schema::attributes::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::insert_into(attributes)
        .values(&form)
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}


async fn show_attribute(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<Attribute>, InternalError>
{
    use schema::attributes::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = attributes
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn update_attribute(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
    Json(form): Json<NewAttribute>,
) -> Result<Json<Attribute>, InternalError>
{
    use schema::attributes::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::update(attributes)
        .filter(id.eq(uuid))
        .set(&form)
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn delete_attribute(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<Attribute>, InternalError>
{
    use schema::attributes::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::delete(attributes)
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/attributes", get(attributes))
        .route("/api/admin/attributes", post(create_attribute))
        .route("/api/admin/attributes/:uuid", get(show_attribute))
        .route("/api/admin/attributes/:uuid", put(update_attribute))
        .route("/api/admin/attributes/:uuid", delete(delete_attribute))
}


fn parse_int_param(params: &HashMap<String, String>, name: &str, default: i64) -> i64 {
    let val = params.get(name).map(|val| val.parse::<i64>().unwrap_or(default)).unwrap_or(default);
    if val <= 0 { 1 } else { val }
}
