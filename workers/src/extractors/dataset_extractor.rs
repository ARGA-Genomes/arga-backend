use std::path::PathBuf;

use chrono::NaiveDateTime;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::Dataset;
use crate::error::Error;
use crate::matchers::source_matcher::{match_sources, SourceRecord, SourceMap};
use super::utils::naive_date_time_from_str;


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    global_id: String,
    name: String,
    source_name: String,
    short_name: Option<String>,
    description: Option<String>,
    url: Option<String>,
    citation: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,

    #[serde(deserialize_with = "naive_date_time_from_str")]
    created: NaiveDateTime,
    #[serde(deserialize_with = "naive_date_time_from_str")]
    updated: NaiveDateTime,
}

impl From<Record> for SourceRecord {
    fn from(value: Record) -> Self {
        Self {
            name: value.source_name,
        }
    }
}


/// Extract datasets from a CSV file
pub fn extract(path: PathBuf, pool: &mut PgPool) -> Result<Vec<Dataset>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let sources = match_sources(&records, pool);
    Ok(extract_datasets(records, &sources))
}


fn extract_datasets(records: Vec<Record>, sources: &SourceMap) -> Vec<Dataset> {
    info!(total=records.len(), "Extracting datasets");

    let mut records: Vec<Dataset> = records.into_par_iter().filter_map(|row| {
        match sources.get(&row.source_name) {
            Some(source) => Some(Dataset {
                id: Uuid::new_v4(),
                source_id: source.id,
                global_id: row.global_id,
                name: row.name,
                short_name: row.short_name,
                description: row.description,
                url: row.url,
                citation: row.citation,
                license: row.license,
                rights_holder: row.rights_holder,
                created_at: row.created.and_utc(),
                updated_at: row.updated.and_utc(),
            }),
            None => None,
        }
    }).collect();

    records.sort_by(|a, b| a.name.cmp(&b.name));
    records.dedup_by(|a, b| a.name.eq(&b.name));

    info!(records=records.len(), "Extracting datasets");
    records
}
