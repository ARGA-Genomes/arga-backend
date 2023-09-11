use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::Ecology;
use crate::error::Error;
use crate::matchers::dataset_matcher::{DatasetMap, match_datasets, DatasetRecord};
use crate::matchers::name_matcher::{match_records, NameRecord, NameMatch};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    global_id: String,
    scientific_name: String,
    canonical_name: Option<String>,
    ecology: String,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: Some(value.scientific_name),
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
pub fn extract(path: PathBuf, pool: &mut PgPool) -> Result<Vec<Ecology>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to a dataset
    let sources = match_datasets(&records, pool);

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(records, pool);
    let regions = extract_regions(&sources, &records)?;
    Ok(regions)
}


fn extract_regions(sources: &DatasetMap, records: &MatchedRecords) -> Result<Vec<Ecology>, Error> {
    info!(total=records.len(), "Extracting ecology");

    let records: Vec<Ecology> = records.into_par_iter().filter_map(|(name, row)| {
        match sources.get(&row.global_id) {
            Some(source) => Some(Ecology {
                id: Uuid::new_v4(),
                dataset_id: source.id,
                name_id: name.id,
                values: extract_ecology_values(&row.ecology),
            }),
            None => None,
        }
    }).collect();

    info!(records=records.len(), "Extracting ecology finished");
    Ok(records)
}

fn extract_ecology_values(values: &str) -> Vec<String> {
    values.split(",").map(|record| record.trim().to_string()).collect()
}
