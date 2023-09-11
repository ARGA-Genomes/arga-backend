use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Regions, RegionType};
use crate::error::{Error, ParseError};
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
    region_type: String,
    regions: String,
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
pub fn extract(path: PathBuf, pool: &mut PgPool) -> Result<Vec<Regions>, Error> {
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


fn extract_regions(sources: &DatasetMap, records: &MatchedRecords) -> Result<Vec<Regions>, Error> {
    info!(total=records.len(), "Extracting regions");

    let regions: Result<Vec<Regions>, ParseError> = records.par_iter().map(|(name, row)| {
        let region_type = extract_region_type(&row.region_type)?;
        let dataset = sources.get(&row.global_id).unwrap();

        Ok(Regions {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            name_id: name.id.clone(),
            values: extract_region_values(&row.regions),
            region_type,
        })
    }).collect();
    let regions = regions?;

    info!(regions=regions.len(), "Extracting regions finished");
    Ok(regions)
}


fn extract_region_type(region_type: &str) -> Result<RegionType, ParseError> {
    match region_type {
        "ibra" => Ok(RegionType::Ibra),
        "imcra" => Ok(RegionType::Imcra),
        "state" => Ok(RegionType::State),
        "drainage_basin" => Ok(RegionType::DrainageBasin),
        _ => Err(ParseError::InvalidValue(region_type.to_string())),
    }
}

fn extract_region_values(values: &str) -> Vec<String> {
    values.split(",").map(|region| region.trim().to_string()).collect()
}
