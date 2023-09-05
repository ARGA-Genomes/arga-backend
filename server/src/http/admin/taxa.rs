use std::collections::HashMap;

use arga_core::models::Taxon;
use axum::extract::{State, Query, Path};
use axum::{Json, Router};
use axum::routing::{get, post, put};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::database::extensions::{pagination, Paginate};
use crate::http::Context;
use crate::http::error::InternalError;
use crate::database::{schema, Database};
use crate::database::models::Name;


#[derive(Debug, Serialize)]
pub struct Page<T> {
    total: i64,
    records: Vec<T>,
}

pub type PageResult<T> = Result<Json<Page<T>>, InternalError>;

impl<T> From<Vec<(T, i64)>> for Page<T> {
    fn from(value: Vec<(T, i64)>) -> Self {
        let page: pagination::Page<T> = value.into();
        Self {
            total: page.total,
            records: page.records,
        }
    }
}


async fn taxa(
    Query(params): Query<HashMap<String, String>>,
    State(database): State<Database>,
) -> PageResult<Name>
{
    use schema::taxa::dsl::*;
    let mut conn = database.pool.get().await?;

    // pagination
    let page = parse_int_param(&params, "page", 1);
    let per_page = parse_int_param(&params, "page_size", 20);

    let mut query = taxa
        .order_by(scientific_name)
        .into_boxed();

    if let Some(search) = params.get("q") {
        let q = format!("%{search}%");
        query = query.filter(canonical_name.ilike(q));
    }

    if let Some(dataset_id) = params.get("dataset_id") {
        let uuid = Uuid::parse_str(dataset_id)?;
        query = query.filter(source.eq(uuid));
    }

    let page = query
        .select((name_id, scientific_name, canonical_name, species_authority))
        .paginate(page)
        .per_page(per_page)
        .load::<(Name, i64)>(&mut conn)
        .await?;

    Ok(Json(page.into()))
}


// #[derive(Debug, Serialize)]
// struct UserTaxaLists {
//     total: usize,
//     records: Vec<UserTaxaList>,
// }

// async fn user_taxa_lists(
//     State(db_provider): State<Database>,
// ) -> Result<Json<UserTaxaLists>, InternalError>
// {
//     use schema::user_taxa_lists::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let records = user_taxa_lists
//         .load::<UserTaxaList>(&mut conn)
//         .await?;

//     Ok(Json(UserTaxaLists {
//         total: records.len(),
//         records,
//     }))
// }

// #[derive(Deserialize, Insertable, AsChangeset, Debug)]
// #[diesel(table_name = schema::user_taxa_lists)]
// struct NewUserTaxaList {
//     name: String,
//     description: Option<String>,
// }

// async fn create_user_taxa_list(
//     State(db_provider): State<Database>,
//     Json(form): Json<NewUserTaxaList>,
// ) -> Result<Json<UserTaxaList>, InternalError>
// {
//     use schema::user_taxa_lists::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let record = diesel::insert_into(user_taxa_lists)
//         .values(&form)
//         .get_result(&mut conn)
//         .await?;

//     Ok(Json(record))
// }

// async fn show_user_taxa_list(
//     Path(uuid): Path<Uuid>,
//     State(db_provider): State<Database>,
// ) -> Result<Json<UserTaxaList>, InternalError>
// {
//     use schema::user_taxa_lists::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let record = user_taxa_lists
//         .filter(id.eq(uuid))
//         .get_result(&mut conn)
//         .await?;

//     Ok(Json(record))
// }

// async fn update_user_taxa_list(
//     Path(uuid): Path<Uuid>,
//     State(db_provider): State<Database>,
//     Json(form): Json<NewUserTaxaList>,
// ) -> Result<Json<UserTaxaList>, InternalError>
// {
//     use schema::user_taxa_lists::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let record = diesel::update(user_taxa_lists)
//         .filter(id.eq(uuid))
//         .set(&form)
//         .get_result(&mut conn)
//         .await?;

//     Ok(Json(record))
// }


// #[derive(Debug, Serialize)]
// struct UserTaxaItems {
//     total: usize,
//     records: Vec<UserTaxon>,
// }

// async fn user_taxa_items(
//     Path(uuid): Path<Uuid>,
//     Query(params): Query<HashMap<String, String>>,
//     State(db_provider): State<Database>,
// ) -> Result<Json<UserTaxaItems>, InternalError>
// {
//     use schema::user_taxa::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let page = parse_int_param(&params, "page", 1);
//     let page_size = parse_int_param(&params, "page_size", 20);
//     let offset = (page - 1) * page_size;

//     let records = user_taxa
//         .filter(taxa_lists_id.eq(uuid))
//         .order_by(scientific_name)
//         .offset(offset)
//         .limit(page_size)
//         .load::<UserTaxon>(&mut conn)
//         .await?;

//     let total: i64 = user_taxa
//         .filter(taxa_lists_id.eq(uuid))
//         .count()
//         .get_result(&mut conn)
//         .await?;

//     Ok(Json(UserTaxaItems {
//         total: total as usize,
//         records,
//     }))
// }

// #[derive(Deserialize, Insertable, AsChangeset, Debug)]
// #[diesel(table_name = schema::user_taxa)]
// struct NewUserTaxon {
//     taxa_lists_id: Option<Uuid>,
//     scientific_name: Option<String>,
//     scientific_name_authorship: Option<String>,
//     canonical_name: Option<String>,
//     specific_epithet: Option<String>,
//     infraspecific_epithet: Option<String>,
//     taxon_rank: Option<String>,
//     name_according_to: Option<String>,
//     name_published_in: Option<String>,
//     taxonomic_status: Option<String>,
//     taxon_remarks: Option<String>,
//     kingdom: Option<String>,
//     phylum: Option<String>,
//     class: Option<String>,
//     order: Option<String>,
//     family: Option<String>,
//     genus: Option<String>,
// }

// async fn create_user_taxon(
//     Path(uuid): Path<Uuid>,
//     State(db_provider): State<Database>,
//     Json(form): Json<NewUserTaxon>,
// ) -> Result<Json<UserTaxon>, InternalError>
// {
//     use schema::user_taxa::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let record = diesel::insert_into(user_taxa)
//         .values(NewUserTaxon { taxa_lists_id: Some(uuid), ..form })
//         .get_result(&mut conn)
//         .await?;

//     Ok(Json(record))
// }

// async fn show_user_taxon(
//     Path(uuid): Path<Uuid>,
//     State(db_provider): State<Database>,
// ) -> Result<Json<UserTaxon>, InternalError>
// {
//     use schema::user_taxa::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let record = user_taxa
//         .filter(id.eq(uuid))
//         .get_result(&mut conn)
//         .await?;

//     Ok(Json(record))
// }

// async fn update_user_taxon(
//     Path(uuid): Path<Uuid>,
//     State(db_provider): State<Database>,
//     Json(form): Json<NewUserTaxon>,
// ) -> Result<Json<UserTaxon>, InternalError>
// {
//     use schema::user_taxa::dsl::*;
//     let mut conn = db_provider.pool.get().await?;

//     let record = diesel::update(user_taxa)
//         .filter(id.eq(uuid))
//         .set(&form)
//         .get_result(&mut conn)
//         .await?;

//     Ok(Json(record))
// }


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/taxa", get(taxa))
        // .route("/api/admin/user_taxa", get(user_taxa_lists))
        // .route("/api/admin/user_taxa", post(create_user_taxa_list))
        // .route("/api/admin/user_taxa/:uuid", get(show_user_taxa_list))
        // .route("/api/admin/user_taxa/:uuid", put(update_user_taxa_list))
        // .route("/api/admin/user_taxa/:uuid/items", get(user_taxa_items))
        // .route("/api/admin/user_taxa/:uuid/items", post(create_user_taxon))
        // .route("/api/admin/user_taxon/:uuid", get(show_user_taxon))
        // .route("/api/admin/user_taxon/:uuid", put(update_user_taxon))
}


fn parse_int_param(params: &HashMap<String, String>, name: &str, default: i64) -> i64 {
    let val = params.get(name).map(|val| val.parse::<i64>().unwrap_or(default)).unwrap_or(default);
    if val <= 0 { 1 } else { val }
}
