use std::collections::HashMap;

use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::routing::get;


use serde::{Serialize, Deserialize};

use crate::http::Context;

use super::error::Error;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Taxa {
    num_found: usize,
    docs: Vec<Record>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    id: String,
    #[serde(rename(deserialize = "occurrenceID"))]
    occurrence_id: String,

    #[serde(rename(deserialize = "genusID"))]
    genus_id: Option<String>,
    #[serde(rename(deserialize = "kingdomID"))]
    kingdom_id: Option<String>,

    scientific_name: Option<String>,
    genus: Option<String>,
    subgenus: Option<String>,
    class: Option<String>,
    kingdom: Option<String>,
    phylum: Option<String>,
    family: Option<String>,

    biome: Option<String>,
    provenance: Option<String>,

    locality: Option<String>,
    state_province: Option<String>,
    country: Option<String>,

    event_date: Option<String>,
    license: Option<String>,
}


pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/search", get(search))
}


async fn search(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Context>,
) -> Result<Json<Taxa>, Error> {
    let query = params.get("q").ok_or(Error::MissingParam("q".to_string()))?;
    let records = state.solr.select::<Taxa>(&query, 10).await?;
    Ok(Json(records))
}
