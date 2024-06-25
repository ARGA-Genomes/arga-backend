use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::models::{Taxon, TaxonomicRank, TaxonomicStatus};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::{Error, ParseError};
use crate::matchers::classification_matcher::{classification_map_scoped, ClassificationMap};
use crate::matchers::dataset_matcher::{dataset_map, match_records, DatasetMatch, DatasetRecord};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(DatasetMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    parent_taxon: Option<String>,
    parent_scientific_name: Option<String>,
    // parent_rank: Option<String>,
    taxon_id: Option<String>,
    entity_id: Option<String>,
    taxon_rank: TaxonomicRank,
    dataset_id: String,
    // accepted_name_usage: Option<String>,
    // original_name_usage: Option<String>,
    scientific_name: String,
    scientific_name_authorship: Option<String>,
    canonical_name: String,
    nomenclatural_code: String,
    taxonomic_status: TaxonomicStatus,

    citation: Option<String>,
    vernacular_name: Option<String>,
    // alternative_names: Option<String>,
    description: Option<String>,
    remarks: Option<String>,
}

impl From<Record> for DatasetRecord {
    fn from(value: Record) -> Self {
        DatasetRecord {
            global_id: value.dataset_id,
        }
    }
}


/// Extract names and taxonomy from a CSV file
pub fn extract(path: &PathBuf, pool: &mut PgPool) -> Result<Vec<Taxon>, Error> {
    let mut records: Vec<Record> = Vec::new();

    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(&path)?;
    for row in reader.deserialize() {
        records.push(row?);
    }

    let datasets = dataset_map(pool)?;
    let mut dataset_ids = records
        .iter()
        .map(|r| datasets.get(&r.dataset_id).map(|d| d.id).unwrap_or_default())
        .collect::<Vec<Uuid>>();
    dataset_ids.dedup();

    let classifications = classification_map_scoped(pool, &dataset_ids)?;
    let taxa = reference_map(&records, &classifications)?;

    let records = match_records(records, &datasets);
    let classifications = extract_classifications(records, &taxa)?;
    Ok(classifications)
}


fn reference_map(records: &Vec<Record>, classifications: &ClassificationMap) -> Result<HashMap<String, Uuid>, Error> {
    let mut map = HashMap::new();

    for (key, val) in classifications.iter() {
        map.insert(key.clone(), val.id.clone());
    }

    // Records and reference others without being in the database. We want to leverage uuid's
    // here and generate them so that referential integrity can be maintained come import time.
    // we also want to make sure to link to existing classifications if they exist so that an
    // incomplete set can still inherit the full tree
    for record in records {
        // attempt to find a match via taxon_id first, falling back to scientific name
        // and finally to canonical name
        let taxon_id = parse_taxon_id_str(&record.taxon_id);
        let id = match classifications.get(&taxon_id.clone().unwrap_or(record.scientific_name.clone())) {
            Some(classification) => classification.id,
            None => Uuid::new_v4(),
            // None => match classifications.get(&record.canonical_name) {
            //     Some(classification) => classification.id,
            //     None => Uuid::new_v4(),
            // },
        };

        map.insert(record.scientific_name.clone(), id.clone());
        // map.insert(record.canonical_name.clone(), id.clone());
        if let Some(taxon_id) = taxon_id {
            map.insert(taxon_id, id.clone());
        }
    }


    Ok(map)
}


fn extract_classifications(records: MatchedRecords, taxa: &HashMap<String, Uuid>) -> Result<Vec<Taxon>, Error> {
    info!(total = records.len(), "Extracting classifications");

    let mut rows = Vec::new();
    for (dataset, record) in records {
        let id = taxa
            .get(&record.scientific_name)
            .ok_or_else(|| ParseError::NotFound(record.scientific_name.clone()))?;

        // the classification map can be used with the scientific_name, canonical_name, or the
        // taxon_id (parent_taxon in our case). this allows us to link to the parent taxon
        // within the database and validate its correctness in the process.
        let parent_id = match record.parent_taxon.or(record.parent_scientific_name) {
            Some(parent) => {
                let parent_id = taxa.get(&parent).ok_or_else(|| ParseError::NotFound(parent))?;
                Some(parent_id.clone())
            }
            None => None,
        };

        rows.push(Taxon {
            id: id.clone(),
            parent_id,
            dataset_id: dataset.id.clone(),
            entity_id: record.entity_id,
            // taxon_id: record.taxon_id.map(parse_taxon_id).unwrap_or(None),
            rank: record.taxon_rank,
            // accepted_name_usage: record.accepted_name_usage,
            // original_name_usage: record.original_name_usage,
            scientific_name: record.scientific_name,
            authorship: record.scientific_name_authorship,
            canonical_name: record.canonical_name,
            nomenclatural_code: record.nomenclatural_code,
            status: record.taxonomic_status,
            citation: record.citation,
            vernacular_names: record.vernacular_name.map(str_to_array),
            // alternative_names: record.alternative_names.map(str_to_array),
            description: record.description,
            remarks: record.remarks,

            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    info!(rows = rows.len(), "Extracting classifications finished");
    Ok(rows)
}


fn str_to_array(value: String) -> Vec<Option<String>> {
    value.split("|").map(|v| Some(String::from(v))).collect()
}


fn parse_taxon_id(value: String) -> Option<i32> {
    value.trim_start_matches("ARGA:BT:").parse::<i32>().ok()
}

fn parse_taxon_id_str(value: &Option<String>) -> Option<String> {
    match value {
        Some(value) => parse_taxon_id(value.clone()).map(|id| id.to_string()),
        None => None,
    }
}
