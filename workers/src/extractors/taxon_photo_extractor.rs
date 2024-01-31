use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use serde::Deserialize;
use tracing::info;

use arga_core::models::TaxonPhoto;

use crate::error::Error;
use crate::matchers::classification_matcher::{ClassificationMatch, ClassificationRecord, classification_map, match_records};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(ClassificationMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    url: String,
    source: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
    priority: Option<i32>,
}

impl From<Record> for ClassificationRecord {
    fn from(value: Record) -> Self {
        ClassificationRecord {
            taxon_id: None,
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}


/// Extract photos from a CSV file
pub fn extract(path: &PathBuf, pool: &mut PgPool) -> Result<Vec<TaxonPhoto>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let classifications = classification_map(pool)?;
    let records = match_records(records, &classifications);
    let photos = extract_photos(records);

    Ok(photos)
}


fn extract_photos(records: MatchedRecords) -> Vec<TaxonPhoto> {
    info!(total=records.len(), "Extracting taxon photos");

    let mut rows = Vec::new();
    for (classification, record) in records {
        rows.push(TaxonPhoto {
            id: uuid::Uuid::new_v4(),
            taxon_id: classification.id,
            url: record.url,
            source: record.source,
            publisher: record.publisher,
            license: record.license,
            rights_holder: record.rights_holder,
            priority: record.priority.unwrap_or(1),
        })
    }

    info!(vernacular_names=rows.len(), "Extracting taxon photos finished");
    rows
}
