use arga_core::schema_gnl;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::database::models::{TaxonomicRank, TaxonomicStatus};
use crate::database::sources::ALA_DATASET_ID;
use crate::database::{Database, schema};
use crate::http::Context;
use crate::http::error::InternalError;


#[derive(Serialize, Debug, Queryable, Selectable)]
#[diesel(table_name = schema::taxa)]
struct TaxonName {
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
    pub rank: TaxonomicRank,
}

#[derive(Deserialize)]
pub struct ParentTaxon {
    /// The scientific name of the parent taxon to retrieve descendants of
    parent: String,
}


async fn classes(State(database): State<Database>) -> Result<Json<Vec<TaxonName>>, InternalError> {
    use schema::{datasets, taxa};
    let mut conn = database.pool.get().await?;

    let records = taxa::table
        .inner_join(datasets::table)
        .select(TaxonName::as_select())
        .filter(taxa::rank.eq(TaxonomicRank::Class))
        .filter(taxa::status.eq(TaxonomicStatus::Accepted))
        .filter(datasets::global_id.eq(ALA_DATASET_ID))
        .order_by(taxa::scientific_name)
        .load::<TaxonName>(&mut conn)
        .await?;

    Ok(Json(records))
}

async fn families(
    Query(params): Query<ParentTaxon>,
    State(database): State<Database>,
) -> Result<Json<Vec<TaxonName>>, InternalError> {
    let records = get_descendants(&database, params.parent, TaxonomicRank::Family).await?;
    Ok(Json(records))
}

async fn genera(
    Query(params): Query<ParentTaxon>,
    State(database): State<Database>,
) -> Result<Json<Vec<TaxonName>>, InternalError> {
    let records = get_descendants(&database, params.parent, TaxonomicRank::Genus).await?;
    Ok(Json(records))
}


async fn species(
    Query(params): Query<ParentTaxon>,
    State(database): State<Database>,
) -> Result<Json<Vec<TaxonName>>, InternalError> {
    let records = get_descendants(&database, params.parent, TaxonomicRank::Species).await?;
    Ok(Json(records))
}


pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/taxa/species", get(species))
        .route("/taxa/genus", get(genera))
        .route("/taxa/family", get(families))
        .route("/taxa/class", get(classes))
}


async fn get_descendants(
    database: &Database,
    parent: String,
    rank: TaxonomicRank,
) -> Result<Vec<TaxonName>, InternalError> {
    use schema::{datasets, taxa};
    use schema_gnl::taxa_dag;

    let mut conn = database.pool.get().await?;

    // the root taxon
    let root = diesel::alias!(taxa as root);

    let records = taxa_dag::table
        .inner_join(root.on(root.field(taxa::id).eq(taxa_dag::id)))
        .inner_join(taxa::table.on(taxa::id.eq(taxa_dag::taxon_id)))
        .inner_join(datasets::table.on(taxa::dataset_id.eq(datasets::id)))
        .select(TaxonName::as_select())
        .filter(root.field(taxa::scientific_name).eq(parent))
        .filter(taxa::rank.eq(rank))
        .filter(taxa::status.eq(TaxonomicStatus::Accepted))
        .filter(datasets::global_id.eq(ALA_DATASET_ID))
        .order_by(taxa::scientific_name)
        .load::<TaxonName>(&mut conn)
        .await?;

    Ok(records)
}
