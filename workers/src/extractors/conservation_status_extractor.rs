use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{ConservationStatus, NameList};
use crate::error::{Error, ParseError};
use crate::matchers::name_matcher::{match_records, NameRecord, NameMatch};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    canonical_name: Option<String>,
    status: String,
    state: Option<String>,
    source: Option<String>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: Some(value.scientific_name),
            canonical_name: value.canonical_name,
        }
    }
}


/// Extract conservation statuses from a CSV file
pub fn extract(path: PathBuf, source: &NameList, pool: &mut PgPool) -> Result<Vec<ConservationStatus>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(records, pool);
    let statuses = extract_statuses(source, records)?;
    Ok(statuses)
}


fn extract_statuses(source: &NameList, records: MatchedRecords) -> Result<Vec<ConservationStatus>, Error> {
    info!(total=records.len(), "Extracting conservation statuses");

    let statuses: Result<Vec<ConservationStatus>, ParseError> = records.into_par_iter().map(|(name, row)| {
        Ok(ConservationStatus {
            id: Uuid::new_v4(),
            name_id: name.id,
            list_id: source.id.clone(),
            status: row.status,
            state: row.state,
            source: row.source,
        })
    }).collect();
    let statuses = statuses?;

    info!(statuses=statuses.len(), "Extracting conservation statuses finished");
    Ok(statuses)
}
