use std::path::PathBuf;

use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;
use arga_core::models::AccessRightsStatus;
use arga_core::models::DataReuseStatus;
use arga_core::models::Source;
use arga_core::models::SourceContentType;

#[derive(Debug, Clone, Deserialize)]
struct Record {
    name: String,
    author: String,
    rights_holder: String,
    access_rights: String,
    license: String,
    reuse_pill: Option<DataReuseStatus>,
    access_pill: Option<AccessRightsStatus>,
    content_type: Option<SourceContentType>,
}

/// Extract sources from a CSV file
pub fn extract(path: PathBuf) -> Result<Vec<Source>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    Ok(extract_source(records))
}

fn extract_source(records: Vec<Record>) -> Vec<Source> {
    info!(total = records.len(), "Extracting sources");

    let mut records: Vec<Source> = records
        .into_par_iter()
        .map(|row| Source {
            id: Uuid::new_v4(),
            name: row.name,
            author: row.author,
            rights_holder: row.rights_holder,
            access_rights: row.access_rights,
            license: row.license,
            reuse_pill: row.reuse_pill,
            access_pill: row.access_pill,
            content_type: row.content_type,
        })
        .collect();

    records.sort_by(|a, b| a.name.cmp(&b.name));
    records.dedup_by(|a, b| a.name.eq(&b.name));

    info!(records = records.len(), "Extracting sources");
    records
}
