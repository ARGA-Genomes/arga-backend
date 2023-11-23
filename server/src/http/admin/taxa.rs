use std::collections::HashMap;

use axum::extract::{State, Query};
use axum::{Json, Router};
use axum::routing::{get, post, put, delete};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::extensions::Paginate;
use crate::http::Context;
use crate::http::error::InternalError;
use crate::database::{schema, Database};
use crate::database::models::{Name, Dataset, Classification, TaxonomicRank, TaxonomicStatus};

use super::common::PageResult;


type ClassificationOptions = HashMap<TaxonomicRank, Vec<ClassificationOption>>;


#[derive(Serialize, Debug)]
struct ClassificationOption {
    pub id: Uuid,
    pub scientific_name: String,
}


#[derive(Deserialize, Debug)]
struct NewClassification {
    pub dataset_id: Uuid,
    pub parent_id: Uuid,
    // pub taxon_id: String,

    pub rank: TaxonomicRank,
    pub accepted_name_usage: String,
    pub original_name_usage: String,
    pub scientific_name_authorship: String,
    pub canonical_name: String,
    pub nomenclatural_code: String,
    pub status: TaxonomicStatus,

    pub citation: Option<String>,
    pub vernacular_names: Option<Vec<String>>,
    pub alternative_names: Option<Vec<String>>,
    pub description: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UpdateClassification {
    pub id: Uuid,
    pub dataset_id: Uuid,

    pub rank: TaxonomicRank,
    pub accepted_name_usage: String,
    pub original_name_usage: String,
    pub scientific_name_authorship: String,
    pub canonical_name: String,
    pub nomenclatural_code: String,
    pub status: TaxonomicStatus,

    pub citation: Option<String>,
    pub vernacular_names: Option<Vec<String>>,
    pub alternative_names: Option<Vec<String>>,
    pub description: Option<String>,
    pub remarks: Option<String>,
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

    if let Some(dataset) = params.get("dataset_id") {
        let uuid = Uuid::parse_str(dataset)?;
        query = query.filter(dataset_id.eq(uuid));
    }

    let page = query
        .select((name_id, scientific_name, canonical_name, species_authority))
        .paginate(page)
        .per_page(per_page)
        .load::<(Name, i64)>(&mut conn)
        .await?;

    Ok(Json(page.into()))
}


async fn classifications(
    Query(params): Query<HashMap<String, String>>,
    State(database): State<Database>,
) -> Result<Json<Vec<Classification>>, InternalError>
{
    use schema::classifications;

    let mut conn = database.pool.get().await?;

    let mut query = classifications::table
        .order_by(classifications::scientific_name)
        .into_boxed();

    if let Some(parent_id) = params.get("parent") {
        let uuid = Uuid::parse_str(parent_id)?;
        query = query.filter(classifications::parent_id.eq(uuid));
    }

    let records = query
        .load::<Classification>(&mut conn)
        .await?;

    Ok(Json(records))
}


async fn get_next_taxon_id(database: &Database) -> Result<i64, InternalError> {
    use schema::classifications::dsl::*;

    let mut conn = database.pool.get().await?;

    let highest_id: String = classifications
        .select(taxon_id)
        .order_by(taxon_id.desc())
        .limit(1)
        .get_result(&mut conn)
        .await?;

    let components = highest_id.split(":");
    let num = str::parse::<i64>(components.last().unwrap()).unwrap();

    return Ok(num + 1);
}


async fn create_classifications(
    State(database): State<Database>,
    Json(form): Json<Vec<NewClassification>>,
) -> Result<Json<Vec<Classification>>, InternalError>
{
    use schema::classifications::dsl::*;

    let mut conn = database.pool.get().await?;

    let taxon_id_number = get_next_taxon_id(&database).await?;

    let mut records = Vec::new();
    for row in form {
        records.push(Classification {
            id: Uuid::new_v4(),
            dataset_id: row.dataset_id,
            parent_id: row.parent_id,
            taxon_id: format!("ARGA:BT:{}", taxon_id_number).to_string(),
            rank: row.rank,
            accepted_name_usage: row.accepted_name_usage,
            original_name_usage: row.original_name_usage,
            scientific_name: format!("{} {}", row.canonical_name, row.scientific_name_authorship),
            scientific_name_authorship: row.scientific_name_authorship,
            canonical_name: row.canonical_name,
            nomenclatural_code: row.nomenclatural_code,
            status: row.status,
            citation: row.citation,
            vernacular_names: None,
            alternative_names: None,
            description: row.description,
            remarks: row.remarks,
        })
    }

    let inserted = diesel::insert_into(classifications)
        .values(records)
        .get_results(&mut conn)
        .await?;

    Ok(Json(inserted))
}


async fn update_classifications(
    State(database): State<Database>,
    Json(form): Json<Vec<UpdateClassification>>,
) -> Result<Json<Vec<Classification>>, InternalError>
{
    use schema::classifications::dsl::*;

    let mut conn = database.pool.get().await?;

    for row in form {
        diesel::update(classifications.filter(id.eq(row.id)))
            .set((
                scientific_name.eq(format!("{} {}", row.canonical_name, row.scientific_name_authorship)),
                dataset_id.eq(row.dataset_id),
                rank.eq(row.rank),
                accepted_name_usage.eq(row.accepted_name_usage),
                original_name_usage.eq(row.original_name_usage),
                scientific_name_authorship.eq(row.scientific_name_authorship),
                canonical_name.eq(row.canonical_name),
                nomenclatural_code.eq(row.nomenclatural_code),
                status.eq(row.status),
                citation.eq(row.citation),
                vernacular_names.eq(row.vernacular_names),
                alternative_names.eq(row.alternative_names),
                description.eq(row.description),
                remarks.eq(row.remarks),
            ))
            .execute(&mut conn)
            .await?;
    }

    Ok(Json(vec![]))
}

async fn delete_classifications(
    State(database): State<Database>,
    Json(form): Json<Vec<Uuid>>,
) -> Result<Json<Vec<Classification>>, InternalError>
{
    use schema::classifications::dsl::*;
    let mut conn = database.pool.get().await?;
    diesel::delete(classifications.filter(id.eq_any(form))).execute(&mut conn).await?;
    Ok(Json(vec![]))
}


async fn classification_options(State(database): State<Database>) -> Result<Json<ClassificationOptions>, InternalError>
{
    use schema::classifications::dsl::*;
    let mut conn = database.pool.get().await?;

    let options = classifications
        .order_by((rank, scientific_name))
        .select((id, rank, scientific_name))
        .load::<(Uuid, TaxonomicRank, String)>(&mut conn)
        .await?;

    let mut groups: ClassificationOptions = HashMap::new();

    for (uuid, taxon_rank, name) in options {
        groups.entry(taxon_rank).or_default().push(ClassificationOption {
            id: uuid,
            scientific_name: name,
        });
    }

    Ok(Json(groups))
}



async fn datasets(State(database): State<Database>) -> Result<Json<Vec<Dataset>>, InternalError> {
    use schema::datasets::dsl::*;
    let mut conn = database.pool.get().await?;
    let dataset = datasets.load::<Dataset>(&mut conn).await?;
    Ok(Json(dataset))
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
        .route("/api/admin/taxa/datasets", get(datasets))
        .route("/api/admin/classification_options", get(classification_options))
        .route("/api/admin/classifications", get(classifications))
        .route("/api/admin/classifications", post(create_classifications))
        .route("/api/admin/classifications", put(update_classifications))
        .route("/api/admin/classifications", delete(delete_classifications))
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
