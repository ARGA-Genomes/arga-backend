use arga_core::models::{Dataset, AttributeCategory, AttributeValueType};
use axum::extract::State;
use axum::{Json, Router};
use axum::routing::{get, post, put, delete};

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use crate::http::Context;
use crate::http::error::InternalError;
use crate::database::{schema, Database};
use crate::database::models::NameAttribute;


#[derive(Deserialize, Debug)]
struct NewAttribute {
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub name: String,
    pub category: AttributeCategory,
    pub value_type: AttributeValueType,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_decimal: Option<BigDecimal>,
    pub value_str: Option<String>,
    pub value_timestamp: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug)]
struct UpdateAttribute {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub name: String,
    pub category: AttributeCategory,
    pub value_type: AttributeValueType,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_decimal: Option<BigDecimal>,
    pub value_str: Option<String>,
    pub value_timestamp: Option<NaiveDateTime>,
}


async fn attributes(State(database): State<Database>) -> Result<Json<Vec<NameAttribute>>, InternalError> {
    use schema::name_attributes::dsl::*;
    let mut conn = database.pool.get().await?;

    let records = name_attributes
        .order_by((dataset_id, name_id, name))
        .load::<NameAttribute>(&mut conn)
        .await?;

    Ok(Json(records))
}

async fn create_attributes(
    State(database): State<Database>,
    Json(form): Json<Vec<NewAttribute>>,
) -> Result<Json<Vec<NameAttribute>>, InternalError>
{
    use schema::name_attributes::dsl::*;

    let mut conn = database.pool.get().await?;

    let mut records = Vec::new();
    for row in form {
        records.push(NameAttribute {
            id: Uuid::new_v4(),
            dataset_id: row.dataset_id,
            name_id: row.name_id,
            name: row.name,
            category: row.category,
            value_type: row.value_type,
            value_bool: row.value_bool,
            value_int: row.value_int,
            value_decimal: row.value_decimal,
            value_str: row.value_str,
            value_timestamp: row.value_timestamp,
        })
    }

    let inserted = diesel::insert_into(name_attributes)
        .values(records)
        .get_results(&mut conn)
        .await?;

    Ok(Json(inserted))
}


async fn update_attributes(
    State(database): State<Database>,
    Json(form): Json<Vec<UpdateAttribute>>,
) -> Result<Json<Vec<NameAttribute>>, InternalError>
{
    use schema::name_attributes::dsl::*;

    let mut conn = database.pool.get().await?;

    for row in form {
        diesel::update(name_attributes.filter(id.eq(row.id)))
            .set((
                dataset_id.eq(row.dataset_id),
                name_id.eq(row.name_id),
                name.eq(row.name),
                category.eq(row.category),
                value_type.eq(row.value_type),
                value_bool.eq(row.value_bool),
                value_int.eq(row.value_int),
                value_decimal.eq(row.value_decimal),
                value_str.eq(row.value_str),
                value_timestamp.eq(row.value_timestamp),
            ))
            .execute(&mut conn)
            .await?;
    }

    Ok(Json(vec![]))
}

async fn delete_attributes(
    State(database): State<Database>,
    Json(form): Json<Vec<Uuid>>,
) -> Result<Json<Vec<Dataset>>, InternalError>
{
    use schema::name_attributes::dsl::*;
    let mut conn = database.pool.get().await?;
    diesel::delete(name_attributes.filter(id.eq_any(form))).execute(&mut conn).await?;
    Ok(Json(vec![]))
}


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/attributes", get(attributes))
        .route("/api/admin/attributes", post(create_attributes))
        .route("/api/admin/attributes", put(update_attributes))
        .route("/api/admin/attributes", delete(delete_attributes))
}
