use std::path::PathBuf;

use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::Name;
use crate::error::Error;
use crate::extractors::utils::decompose_scientific_name;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    scientific_name: String,
    canonical_name: Option<String>,
    authority: Option<String>,
}


/// Extract names from a CSV file
pub fn extract(path: &PathBuf) -> Result<Vec<Name>, Error> {
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
        // if certain fields making up a scientific name can't be found try
        // to extract it from the scientific name
        let decomposed = decompose_scientific_name(&row.scientific_name);

        let species_authority = match &row.authority {
            Some(authority) => Some(authority.clone()),
            None => decomposed.clone().map(|v| v.authority)
        };

        let canonical_name = match &row.canonical_name {
            Some(canonical_name) => Some(canonical_name.clone()),
            None => decomposed.map(|v| v.canonical_name())
        };

        Name {
            id: Uuid::new_v4(),
            scientific_name: row.scientific_name.clone(),
            canonical_name: canonical_name.unwrap_or_else(|| row.scientific_name.clone()),
            authorship: species_authority,
        }
    }).collect::<Vec<Name>>();

    info!(names=names.len(), "Extracting names finished");
    names
}
