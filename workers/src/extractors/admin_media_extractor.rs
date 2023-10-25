use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;

use arga_core::models::AdminMedia;
use uuid::Uuid;
use crate::error::Error;
use crate::matchers::name_matcher::{match_records, NameRecord, NameMatch};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    url: String,
    reference_url: Option<String>,
    source: Option<String>,
    title: Option<String>,
    description: Option<String>,
    creator: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}

/// Extract admin media from a CSV file
pub fn extract(path: PathBuf, image_source: &str, pool: &mut PgPool) -> Result<Vec<AdminMedia>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(records, pool);
    let attrs = extract_media(image_source, records)?;
    Ok(attrs)
}


fn extract_media(image_source: &str, records: MatchedRecords) -> Result<Vec<AdminMedia>, Error> {
    info!(total=records.len(), "Extracting admin media");

    let media = records.into_par_iter().map(|(name, row)| {
        AdminMedia {
            id: Uuid::new_v4(),
            name_id: name.id.clone(),
            image_source: image_source.to_string(),
            width: row.width,
            height: row.height,
            url: row.url,
            reference_url: row.reference_url,
            title: row.title,
            description: row.description,
            source: row.source,
            creator: row.creator,
            publisher: row.publisher,
            license: row.license,
            rights_holder: row.rights_holder,
        }
    }).collect::<Vec<AdminMedia>>();

    info!(media=media.len(), "Extracting admin media finished");
    Ok(media)
}
