use std::path::PathBuf;

use chrono::NaiveDateTime;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::IndigenousKnowledge;
use crate::error::Error;
use crate::matchers::dataset_matcher::{match_datasets, DatasetRecord, DatasetMap};
use crate::matchers::name_matcher::{match_records, NameRecord, NameMatch};
use super::utils::naive_date_time_from_str;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    global_id: String,
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    vernacular_name: String,
    food_use: String,
    medicinal_use: String,
    cultural_connection: String,
    source_url: Option<String>,

    #[serde(deserialize_with = "naive_date_time_from_str")]
    last_updated: NaiveDateTime,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}

impl From<Record> for DatasetRecord {
    fn from(value: Record) -> Self {
        Self {
            global_id: value.global_id,
        }
    }
}


/// Extract regions from a CSV file
pub fn extract(path: PathBuf, pool: &mut PgPool) -> Result<Vec<IndigenousKnowledge>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to a dataset
    let sources = match_datasets(&records, pool);

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(records, pool);
    let records = extract_indigenous_knowledge(&sources, records)?;
    Ok(records)
}


fn extract_indigenous_knowledge(sources: &DatasetMap, records: MatchedRecords) -> Result<Vec<IndigenousKnowledge>, Error> {
    info!(total=records.len(), "Extracting indigenous knowledge");

    let records: Vec<IndigenousKnowledge> = records.into_par_iter().filter_map(|(name, row)| {
        match sources.get(&row.global_id) {
            Some(source) => Some(IndigenousKnowledge {
                id: Uuid::new_v4(),
                dataset_id: source.id,
                name_id: name.id,
                name: row.vernacular_name,
                food_use: row.food_use.to_lowercase() == "true",
                medicinal_use: row.medicinal_use.to_lowercase() == "true",
                cultural_connection: row.cultural_connection.to_lowercase() == "true",
                last_updated: row.last_updated.and_utc(),
                source_url: row.source_url,
            }),
            None => None,
        }
    }).collect();

    info!(records=records.len(), "Extracting indigenous knowledge finished");
    Ok(records)
}
