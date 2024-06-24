use std::path::PathBuf;

use arga_core::models::{TaxonomicAct, TaxonomicActType};
use chrono::Utc;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;
use crate::matchers::classification_matcher::{classification_map_scoped, ClassificationMap};
use crate::matchers::dataset_matcher::dataset_map;


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    entity_id: String,
    taxon: String,
    accepted_taxon: Option<String>,
    act: TaxonomicActType,
    source_url: Option<String>,
    dataset_id: String,
}


/// Extract acts from a CSV file
pub fn extract(path: &PathBuf, pool: &mut PgPool) -> Result<Vec<TaxonomicAct>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let datasets = dataset_map(pool)?;
    let mut dataset_ids = records
        .iter()
        .map(|r| datasets.get(&r.dataset_id).map(|d| d.id).unwrap_or_default())
        .collect::<Vec<Uuid>>();
    dataset_ids.dedup();

    let classifications = classification_map_scoped(pool, &dataset_ids)?;
    let taxa = extract_acts(&records, &classifications);
    Ok(taxa)
}


fn extract_acts(records: &Vec<Record>, classifications: &ClassificationMap) -> Vec<TaxonomicAct> {
    info!(total = records.len(), "Extracting taxonomic acts");

    let taxa = records
        .iter()
        .map(|row| {
            let taxon = row.taxon.trim().to_string();
            let taxon = classifications.get(&taxon).unwrap();

            let accepted_taxon = match &row.accepted_taxon {
                Some(taxon) => classifications.get(taxon).clone(),
                None => None,
            };

            TaxonomicAct {
                id: Uuid::new_v4(),
                entity_id: row.entity_id.clone(),
                taxon_id: taxon.id,
                accepted_taxon_id: accepted_taxon.map(|t| t.id),
                act: row.act.clone(),
                source_url: row.source_url.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        })
        .collect::<Vec<TaxonomicAct>>();

    info!(taxa = taxa.len(), "Extracting taxa finished");
    taxa
}
