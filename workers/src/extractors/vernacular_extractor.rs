use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::schema;

use crate::error::Error;
use crate::matchers::name_matcher::{NameRecord, NameMatch};
use crate::matchers::vernacular_matcher::{self, VernacularMatch, VernacularRecord};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedVernacular = Vec<(VernacularMatch, NameMatch, Record)>;


#[derive(Debug, Insertable, Queryable, Deserialize)]
#[diesel(table_name = schema::vernacular_names)]
pub struct VernacularName {
    vernacular_name: String,
    language: Option<String>,
}

#[derive(Debug, Insertable, Queryable, Deserialize)]
#[diesel(table_name = schema::name_vernacular_names)]
pub struct VernacularNameLink {
    name_id: Uuid,
    vernacular_name_id: i64,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    canonical_name: Option<String>,
    vernacular_name: String,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}

impl From<Record> for VernacularRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            vernacular_name: value.vernacular_name,
            language: Some("en".to_string()),
        }
    }
}


/// Extract vernacular names from a CSV file
pub fn extract(path: &PathBuf) -> Result<Vec<VernacularName>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let names = extract_vernacular(&records);
    Ok(names)
}

/// Extract vernacular name links from a CSV file
pub fn extract_links(path: &PathBuf, pool: &mut PgPool) -> Result<Vec<VernacularNameLink>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to names and vernacular names in the database. this will filter out
    // any names that could not be matched and is the reason why we need to also import
    // vernacular names first
    let vernacular_records = vernacular_matcher::match_records(records, pool);
    let links = extract_vernacular_links(&vernacular_records);
    Ok(links)
}


fn extract_vernacular(records: &Vec<Record>) -> Vec<VernacularName> {
    info!(total=records.len(), "Extracting vernacular names");

    let vernacular_names = records.par_iter().map(|row| {
        VernacularName {
            vernacular_name: row.vernacular_name.clone(),
            language: Some("en".to_string()),
        }
    }).collect::<Vec<VernacularName>>();

    info!(vernacular_names=vernacular_names.len(), "Extracting vernacular names finished");
    vernacular_names
}


fn extract_vernacular_links(records: &MatchedVernacular) -> Vec<VernacularNameLink> {
    info!(total=records.len(), "Extracting vernacular name links");

    let links = records.par_iter().map(|(vernacular, name, _row)| {
        VernacularNameLink {
            name_id: name.id.clone(),
            vernacular_name_id: vernacular.id,
        }
    }).collect::<Vec<VernacularNameLink>>();

    info!(links=links.len(), "Extracting vernacular name links finished");
    links
}
