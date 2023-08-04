use std::path::PathBuf;

use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::database::models::Name;
use crate::workers::error::Error;
use crate::workers::extractors::utils::extract_authority;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    canonical_name: Option<String>,
    authority: Option<String>,
}


/// Extract names from a CSV file
pub fn extract(path: PathBuf) -> Result<Vec<Name>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let names = extract_names(&records);
    Ok(names)
}


fn extract_names(records: &Vec<Record>) -> Vec<Name> {
    info!(total=records.len(), "Extracting names");

    let names = records.par_iter().map(|row| {
        // fallback to extracting the authority from the scientific name if a species value isn't present
        let species_authority = match &row.authority {
            Some(authority) => Some(authority.clone()),
            None => extract_authority(&row.canonical_name, &Some(row.scientific_name.clone())),
        };

        Name {
            id: Uuid::new_v4(),
            scientific_name: row.scientific_name.clone(),
            canonical_name: row.canonical_name.clone(),
            authorship: species_authority,
        }
    }).collect::<Vec<Name>>();

    info!(names=names.len(), "Extracting names finished");
    names
}
