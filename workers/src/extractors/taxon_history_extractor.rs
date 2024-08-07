use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::models::{Dataset, TaxonHistory};
use arga_core::schema;
use chrono::{DateTime, Utc};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use super::utils::date_time_from_str_opt;
use crate::error::Error;
use crate::matchers::taxon_matcher::{self, TaxonMatch};

type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Clone, Deserialize)]
struct Record {
    acted_on: String,
    scientific_name: String,
    // taxonomic_status: String,
    publication: Option<String>,
    source_url: Option<String>,
    #[serde(deserialize_with = "date_time_from_str_opt")]
    created_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "date_time_from_str_opt")]
    updated_at: Option<DateTime<Utc>>,
    entity_id: Option<String>,
}

/// Extract simple taxonomic history from a CSV file
pub fn extract(path: &PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<Vec<TaxonHistory>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let taxa = taxon_matcher::taxa_map(dataset, pool)?;
    let publications = publications_map(pool)?;
    let history = extract_history(dataset, &records, &taxa, &publications);
    Ok(history)
}

fn extract_history(
    dataset: &Dataset,
    records: &Vec<Record>,
    taxa: &HashMap<String, TaxonMatch>,
    publications: &HashMap<String, Uuid>,
) -> Vec<TaxonHistory> {
    info!(total = records.len(), "Extracting taxon history");

    let history = records
        .par_iter()
        .map(|row| {
            let acted_on = taxa.get(&row.acted_on);
            let taxon_id = taxa.get(&row.scientific_name);
            let publication_id = match &row.publication {
                Some(publication) => publications.get(publication).map(|id| id.clone()),
                None => None,
            };

            match (acted_on, taxon_id) {
                (Some(acted_on), Some(taxon_id)) => Some(TaxonHistory {
                    id: Uuid::new_v4(),
                    acted_on: acted_on.id,
                    taxon_id: taxon_id.id,
                    publication_id,
                    source_url: row.source_url.clone(),
                    dataset_id: dataset.id.clone(),
                    // TODO: default to current timestamp since the schema defaults to that
                    // anyway. we probably want to have separate record timestamp columns
                    created_at: row.created_at.unwrap_or_else(|| Utc::now()),
                    updated_at: row.updated_at.unwrap_or_else(|| Utc::now()),
                    entity_id: row.entity_id.clone(),
                }),
                _ => None,
            }
        })
        .collect::<Vec<Option<TaxonHistory>>>();

    let history: Vec<TaxonHistory> = history.into_iter().filter_map(|r| r).collect();

    info!(history = history.len(), "Extracting taxon history finished");
    history
}

fn publications_map(pool: &mut PgPool) -> Result<HashMap<String, Uuid>, Error> {
    use schema::name_publications::dsl::*;
    let mut conn = pool.get()?;

    let records = name_publications
        .select((id, citation.assume_not_null()))
        .filter(citation.is_not_null())
        .load::<(Uuid, String)>(&mut conn)?;

    let mut map = HashMap::new();
    for (pub_id, pub_citation) in records {
        map.insert(pub_citation, pub_id);
    }

    Ok(map)
}
