use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use serde::Deserialize;
use tracing::info;

use arga_core::models::{VernacularName, Dataset};

use crate::error::Error;
use crate::matchers::name_matcher::{NameRecord, NameMatch, name_map, match_records_mapped};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    vernacular_name: String,
    citation: Option<String>,
    source_url: Option<String>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        NameRecord {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}


/// Extract vernacular names from a CSV file
pub fn extract(path: &PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<Vec<VernacularName>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let names = name_map(pool)?;
    let records = match_records_mapped(records, &names)?;

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let vernacular = extract_vernacular(dataset, records);
    Ok(vernacular)
}


fn extract_vernacular(dataset: &Dataset, records: MatchedRecords) -> Vec<VernacularName> {
    info!(total=records.len(), "Extracting vernacular names");

    let mut rows = Vec::new();
    for (name, record) in records {
        rows.push(VernacularName {
            id: uuid::Uuid::new_v4(),
            dataset_id: dataset.id,
            name_id: name.id,
            vernacular_name: record.vernacular_name,
            citation: record.citation,
            source_url: record.source_url,
        })
    }

    info!(vernacular_names=rows.len(), "Extracting vernacular names finished");
    rows
}
