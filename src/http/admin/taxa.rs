use std::collections::HashMap;

use axum::extract::{State, Query, Path};
use axum::{Json, Router};
use axum::routing::{get, post, delete, put};

use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::{BoxedSelectStatement, FromClause};
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};
use uuid::Uuid;


use crate::http::Context;
use crate::http::error::InternalError;
use crate::database::{schema, schema_gnl, Database};
use crate::database::models::{UserTaxaList, UserTaxon, ArgaTaxon, AttributeDataType};


#[derive(Debug, Serialize)]
struct TaxaList {
    total: usize,
    records: Vec<ArgaTaxon>,
}

async fn taxa(
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<TaxaList>, InternalError>
{
    use schema_gnl::gnl::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    // pagination
    let page = parse_int_param(&params, "page", 1);
    let page_size = parse_int_param(&params, "page_size", 20);
    let offset = (page - 1) * page_size;

    let mut query = gnl
        .order_by(scientific_name)
        .offset(offset)
        .limit(page_size)
        .into_boxed();

    let mut total = gnl
        .into_boxed();

    // filters
    if let Some(filter_source) = params.get("source") {
        query = query.filter(source.eq(filter_source));
        total = total.filter(source.eq(filter_source));

        match parse_uuid(&params, "taxa_lists_id") {
            Some(list_uuid) => {
                query = query.filter(taxa_lists_id.eq(list_uuid));
                total = total.filter(taxa_lists_id.eq(list_uuid));
            },
            None => {
                query = query.filter(taxa_lists_id.is_null());
                total = total.filter(taxa_lists_id.is_null());
            }
        };
    }

    if let Some(search) = params.get("q") {
        let q = format!("%{search}%");
        query = query.filter(canonical_name.ilike(q.clone()));
        total = total.filter(canonical_name.ilike(q));
    }

    let records = query.load::<ArgaTaxon>(&mut conn).await?;
    let total: i64 = total.count().get_result(&mut conn).await?;

    Ok(Json(TaxaList {
        total: total as usize,
        records,
    }))
}


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct EntityAttribute<T> {
    pub id: Uuid,
    pub data_type: AttributeDataType,
    pub name: String,
    pub value: T,
}


async fn taxon(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<Vec<serde_json::Value>>, InternalError>
{
    let mut conn = db_provider.pool.get().await?;

    // TODO: create a view with the EAV table joined and converting the value to json
    // as this will be a common way to get attribute values
    use schema::objects::dsl as objects;
    use schema::attributes::dsl as attributes;
    use schema::object_values_string::dsl as strings;
    use schema::object_values_text::dsl as texts;
    use schema::object_values_array::dsl as arrays;

    let strings = objects::objects
        .inner_join(attributes::attributes.on(objects::attribute_id.eq(attributes::id)))
        .inner_join(strings::object_values_string.on(objects::value_id.eq(strings::id)))
        .select((objects::id, attributes::data_type, attributes::name, strings::value))
        .filter(objects::entity_id.eq(uuid))
        .load::<EntityAttribute<String>>(&mut conn)
        .await?;

    let texts = objects::objects
        .inner_join(attributes::attributes.on(objects::attribute_id.eq(attributes::id)))
        .inner_join(texts::object_values_text.on(objects::value_id.eq(texts::id)))
        .select((objects::id, attributes::data_type, attributes::name, texts::value))
        .filter(objects::entity_id.eq(uuid))
        .load::<EntityAttribute<String>>(&mut conn)
        .await?;

    let arrays = objects::objects
        .inner_join(attributes::attributes.on(objects::attribute_id.eq(attributes::id)))
        .inner_join(arrays::object_values_array.on(objects::value_id.eq(arrays::id)))
        .select((objects::id, attributes::data_type, attributes::name, arrays::value))
        .filter(objects::entity_id.eq(uuid))
        .load::<EntityAttribute<Vec<Option<String>>>>(&mut conn)
        .await?;

    let mut attrs = Vec::new();
    attrs.append(&mut strings.iter().map(|v| serde_json::json!(v)).collect());
    attrs.append(&mut texts.iter().map(|v| serde_json::json!(v)).collect());
    attrs.append(&mut arrays.iter().map(|v| serde_json::json!(v)).collect());

    Ok(Json(attrs))
}


#[derive(Debug, Serialize)]
struct UserTaxaLists {
    total: usize,
    records: Vec<UserTaxaList>,
}

async fn user_taxa_lists(
    State(db_provider): State<Database>,
) -> Result<Json<UserTaxaLists>, InternalError>
{
    use schema::user_taxa_lists::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let records = user_taxa_lists
        .load::<UserTaxaList>(&mut conn)
        .await?;

    Ok(Json(UserTaxaLists {
        total: records.len(),
        records,
    }))
}

#[derive(Deserialize, Insertable, AsChangeset, Debug)]
#[diesel(table_name = schema::user_taxa_lists)]
struct NewUserTaxaList {
    name: String,
    description: Option<String>,
}

async fn create_user_taxa_list(
    State(db_provider): State<Database>,
    Json(form): Json<NewUserTaxaList>,
) -> Result<Json<UserTaxaList>, InternalError>
{
    use schema::user_taxa_lists::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::insert_into(user_taxa_lists)
        .values(&form)
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn show_user_taxa_list(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<UserTaxaList>, InternalError>
{
    use schema::user_taxa_lists::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = user_taxa_lists
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn update_user_taxa_list(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
    Json(form): Json<NewUserTaxaList>,
) -> Result<Json<UserTaxaList>, InternalError>
{
    use schema::user_taxa_lists::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::update(user_taxa_lists)
        .filter(id.eq(uuid))
        .set(&form)
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn delete_user_taxa_list(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<UserTaxaList>, InternalError>
{
    use schema::user_taxa_lists::dsl::*;
    use schema::user_taxa::dsl as user_taxa;
    use schema::object_values_string as obj_strings;
    use schema::object_values_text as obj_text;
    use schema::object_values_integer as obj_integers;
    use schema::object_values_boolean as obj_booleans;
    use schema::object_values_timestamp as obj_timestamps;
    use schema::object_values_array as obj_arrays;
    use schema::objects as obj;

    let mut conn = db_provider.pool.get().await?;


    // delete all the values associated with taxa on the taxa list
    diesel::delete(obj_strings::table).filter(obj_strings::id.eq_any(with_taxa_list(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_text::table).filter(obj_text::id.eq_any(with_taxa_list(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_integers::table).filter(obj_integers::id.eq_any(with_taxa_list(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_booleans::table).filter(obj_booleans::id.eq_any(with_taxa_list(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_timestamps::table).filter(obj_timestamps::id.eq_any(with_taxa_list(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_arrays::table).filter(obj_arrays::id.eq_any(with_taxa_list(&uuid))).execute(&mut conn).await?;

    // delete the taxon object through table eav entries
    use schema_gnl::user_taxa_objects::dsl::{user_taxa_objects, object_id, taxa_lists_id};
    let list_objects = user_taxa_objects.select(object_id).filter(taxa_lists_id.eq(uuid)).into_boxed();
    diesel::delete(obj::table).filter(obj::id.eq_any(list_objects)).execute(&mut conn).await?;

    diesel::delete(user_taxa::user_taxa)
        .filter(user_taxa::taxa_lists_id.eq(uuid))
        .execute(&mut conn)
        .await?;

    let record = diesel::delete(user_taxa_lists)
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}


fn with_taxa_list(uuid: &Uuid) -> BoxedSelectStatement<diesel::sql_types::Uuid, FromClause<schema_gnl::user_taxa_objects::table>, Pg> {
    use schema_gnl::user_taxa_objects::dsl::*;
    user_taxa_objects.select(value_id).filter(taxa_lists_id.eq(uuid)).into_boxed()
}



#[derive(Debug, Serialize)]
struct UserTaxaItems {
    total: usize,
    records: Vec<UserTaxon>,
}

async fn user_taxa_items(
    Path(uuid): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<UserTaxaItems>, InternalError>
{
    use schema::user_taxa::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let page = parse_int_param(&params, "page", 1);
    let page_size = parse_int_param(&params, "page_size", 20);
    let offset = (page - 1) * page_size;

    let records = user_taxa
        .filter(taxa_lists_id.eq(uuid))
        .order_by(scientific_name)
        .offset(offset)
        .limit(page_size)
        .load::<UserTaxon>(&mut conn)
        .await?;

    let total: i64 = user_taxa
        .filter(taxa_lists_id.eq(uuid))
        .count()
        .get_result(&mut conn)
        .await?;

    Ok(Json(UserTaxaItems {
        total: total as usize,
        records,
    }))
}

#[derive(Deserialize, Insertable, AsChangeset, Debug)]
#[diesel(table_name = schema::user_taxa)]
struct NewUserTaxon {
    taxa_lists_id: Option<Uuid>,
    scientific_name: Option<String>,
    scientific_name_authorship: Option<String>,
    canonical_name: Option<String>,
    specific_epithet: Option<String>,
    infraspecific_epithet: Option<String>,
    taxon_rank: Option<String>,
    name_according_to: Option<String>,
    name_published_in: Option<String>,
    taxonomic_status: Option<String>,
    taxon_remarks: Option<String>,
    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    genus: Option<String>,
}

async fn create_user_taxon(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
    Json(form): Json<NewUserTaxon>,
) -> Result<Json<UserTaxon>, InternalError>
{
    use schema::user_taxa::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::insert_into(user_taxa)
        .values(NewUserTaxon { taxa_lists_id: Some(uuid), ..form })
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn show_user_taxon(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<UserTaxon>, InternalError>
{
    use schema::user_taxa::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = user_taxa
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn update_user_taxon(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
    Json(form): Json<NewUserTaxon>,
) -> Result<Json<UserTaxon>, InternalError>
{
    use schema::user_taxa::dsl::*;
    let mut conn = db_provider.pool.get().await?;

    let record = diesel::update(user_taxa)
        .filter(id.eq(uuid))
        .set(&form)
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}

async fn delete_user_taxon(
    Path(uuid): Path<Uuid>,
    State(db_provider): State<Database>,
) -> Result<Json<UserTaxon>, InternalError>
{
    use schema::user_taxa::dsl::*;
    use schema::object_values_string as obj_strings;
    use schema::object_values_text as obj_text;
    use schema::object_values_integer as obj_integers;
    use schema::object_values_boolean as obj_booleans;
    use schema::object_values_timestamp as obj_timestamps;
    use schema::object_values_array as obj_arrays;
    use schema::objects as obj;

    let mut conn = db_provider.pool.get().await?;

    // delete all the values associated with the taxon
    diesel::delete(obj_strings::table).filter(obj_strings::id.eq_any(with_taxon(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_text::table).filter(obj_text::id.eq_any(with_taxon(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_integers::table).filter(obj_integers::id.eq_any(with_taxon(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_booleans::table).filter(obj_booleans::id.eq_any(with_taxon(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_timestamps::table).filter(obj_timestamps::id.eq_any(with_taxon(&uuid))).execute(&mut conn).await?;
    diesel::delete(obj_arrays::table).filter(obj_arrays::id.eq_any(with_taxon(&uuid))).execute(&mut conn).await?;

    // delete the taxon object through table eav entries
    diesel::delete(obj::table).filter(obj::entity_id.eq(uuid)).execute(&mut conn).await?;

    // delete the user taxon
    let record = diesel::delete(user_taxa)
        .filter(id.eq(uuid))
        .get_result(&mut conn)
        .await?;

    Ok(Json(record))
}


fn with_taxon(uuid: &Uuid) -> BoxedSelectStatement<diesel::sql_types::Uuid, FromClause<schema::objects::table>, Pg> {
    use schema::objects::dsl::*;
    objects.select(value_id).filter(entity_id.eq(uuid)).into_boxed()
}


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/taxa", get(taxa))
        .route("/api/admin/taxa/:uuid", get(taxon))
        .route("/api/admin/user_taxa", get(user_taxa_lists))
        .route("/api/admin/user_taxa", post(create_user_taxa_list))
        .route("/api/admin/user_taxa/:uuid", get(show_user_taxa_list))
        .route("/api/admin/user_taxa/:uuid", put(update_user_taxa_list))
        .route("/api/admin/user_taxa/:uuid", delete(delete_user_taxa_list))
        .route("/api/admin/user_taxa/:uuid/items", get(user_taxa_items))
        .route("/api/admin/user_taxa/:uuid/items", post(create_user_taxon))
        .route("/api/admin/user_taxon/:uuid", get(show_user_taxon))
        .route("/api/admin/user_taxon/:uuid", put(update_user_taxon))
        .route("/api/admin/user_taxon/:uuid", delete(delete_user_taxon))
}


fn parse_int_param(params: &HashMap<String, String>, name: &str, default: i64) -> i64 {
    let val = params.get(name).map(|val| val.parse::<i64>().unwrap_or(default)).unwrap_or(default);
    if val <= 0 { 1 } else { val }
}

fn parse_uuid(params: &HashMap<String, String>, list_id: &str) -> Option<Uuid> {
    if let Some(val) = params.get(list_id) {
        match Uuid::parse_str(val) {
            Ok(uuid) => Some(uuid),
            Err(_) => None,
        }
    } else {
        None
    }
}
