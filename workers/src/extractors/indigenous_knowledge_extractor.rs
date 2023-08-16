use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{IndigenousKnowledge, Dataset};
use crate::error::{Error, ParseError};
use crate::matchers::name_matcher::{match_records, NameRecord, NameMatch};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    full_name: String,
    canonical_name: String,
    food_use: String,
    medicinal_use: String,
    cultural_connection: String,
    last_updated: String,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.canonical_name,
            canonical_name: None,
        }
    }
}


/// Extract regions from a CSV file
pub fn extract(path: PathBuf, source: &Dataset, pool: &mut PgPool) -> Result<Vec<IndigenousKnowledge>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(records, pool);
    let records = extract_indigenous_knowledge(source, &records)?;
    Ok(records)
}


fn extract_indigenous_knowledge(source: &Dataset, records: &MatchedRecords) -> Result<Vec<IndigenousKnowledge>, Error> {
    info!(total=records.len(), "Extracting indigenous knowledge");

    let records: Result<Vec<IndigenousKnowledge>, ParseError> = records.par_iter().map(|(name, row)| {
        Ok(IndigenousKnowledge {
            id: Uuid::new_v4(),
            dataset_id: source.id.clone(),
            name_id: name.id.clone(),
            name: row.full_name.clone(),
            food_use: row.food_use.to_lowercase() == "true",
            medicinal_use: row.medicinal_use.to_lowercase() == "true",
            cultural_connection: row.cultural_connection.to_lowercase() == "true",
            last_updated: chrono::NaiveDateTime::parse_from_str(&row.last_updated, "%Y-%m-%dT%H:%M:%SZ").unwrap(),
        })
    }).collect();
    let records = records?;

    info!(records=records.len(), "Extracting indigenous knowledge finished");
    Ok(records)
}
