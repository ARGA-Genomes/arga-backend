use std::path::PathBuf;

use arga_core::models::{Dataset, NamePublication};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use super::utils::{date_time_from_str_opt, try_i32_opt};
use crate::error::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    citation: Option<String>,
    #[serde(deserialize_with = "try_i32_opt")]
    published_year: Option<i32>,
    source_url: Option<String>,
    type_citation: Option<String>,

    #[serde(deserialize_with = "date_time_from_str_opt")]
    record_created_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "date_time_from_str_opt")]
    record_updated_at: Option<DateTime<Utc>>,
}


/// Extract name publications from a CSV file
pub fn extract(path: &PathBuf, dataset: &Dataset) -> Result<Vec<NamePublication>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let publications = extract_publications(dataset, &records);
    Ok(publications)
}


fn extract_publications(dataset: &Dataset, records: &Vec<Record>) -> Vec<NamePublication> {
    info!(total = records.len(), "Extracting name publications");

    let publications: Vec<NamePublication> = records
        .par_iter()
        .map(|row| NamePublication {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            citation: row.citation.clone(),
            published_year: row.published_year,
            source_url: row.source_url.clone(),
            type_citation: row.type_citation.clone(),
            record_created_at: row.record_created_at.clone(),
            record_updated_at: row.record_updated_at.clone(),
        })
        .collect();
    info!(publications = publications.len(), "Extracting publications finished");
    publications
}
