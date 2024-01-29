use std::collections::HashMap;
use std::path::PathBuf;

use chrono::Utc;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{TaxonHistory, Dataset};
use crate::error::Error;
use crate::matchers::classification_matcher::classification_map;
use crate::matchers::taxon_matcher::{self, TaxonRecord, TaxonMatch};


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    new_scientific_name: String,
    old_scientific_name: String,
    changed_by: Option<String>,
    reason: Option<String>,
}


/// Extract simple taxonomic history from a CSV file
pub fn extract(path: &PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<Vec<TaxonHistory>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // combine valid names and synonyms so that we can grab the uuid
    // for each when building the history
    let mut all_names = Vec::new();
    for record in &records {
        all_names.push(TaxonRecord { scientific_name: record.new_scientific_name.clone() });
        all_names.push(TaxonRecord { scientific_name: record.old_scientific_name.clone() });
    }

    let classifications = classification_map(pool);

    // let taxa = taxon_matcher::match_taxa(dataset, &all_names, pool);
    let history = extract_history(&records, &taxa);
    Ok(history)
}


fn extract_history(dataset: &Dataset, records: &Vec<Record>, taxa: &HashMap<String, TaxonMatch>) -> Vec<TaxonHistory> {
    info!(total=records.len(), "Extracting taxon history");

    let history = records.par_iter().map(|row| {
        let old_taxon_id = taxa.get(&row.old_scientific_name);
        let new_taxon_id = taxa.get(&row.new_scientific_name);

        match (old_taxon_id, new_taxon_id) {
            (Some(old_taxon_id), Some(new_taxon_id)) => Some(TaxonHistory {
                id: Uuid::new_v4(),
                old_taxon_id: old_taxon_id.id,
                new_taxon_id: new_taxon_id.id,
                dataset_id: dataset.id.clone(),
                created_at: Utc::now(),
            }),
            _ => None,
        }
    }).collect::<Vec<Option<TaxonHistory>>>();

    let history: Vec<TaxonHistory> = history.into_iter().filter_map(|r| r).collect();

    info!(history=history.len(), "Extracting taxon history finished");
    history
}
