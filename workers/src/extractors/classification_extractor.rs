use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{TaxonomicStatus, Classification, TaxonomicRank};
use crate::error::{Error, ParseError};
use crate::matchers::classification_matcher::{classification_map, ClassificationMap};
use crate::matchers::dataset_matcher::{DatasetRecord, match_records, dataset_map, DatasetMatch};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(DatasetMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    parent_taxon: Option<String>,

    taxon_id: String,
    taxon_rank: String,
    dataset_id: String,
    accepted_name_usage: String,
    original_name_usage: String,
    scientific_name: String,
    scientific_name_authorship: String,
    canonical_name: String,
    nomenclatural_code: String,
    taxonomic_status: String,

    citation: Option<String>,
    vernacular_name: Option<String>,
    alternative_names: Option<String>,
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
pub fn extract(path: &PathBuf, pool: &mut PgPool) -> Result<Vec<Classification>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let datasets = dataset_map(pool)?;
    let classifications = classification_map(pool)?;

    let records = match_records(records, &datasets);
    let classifications = extract_classifications(records, &classifications)?;
    Ok(classifications)
}


fn extract_classifications(records: MatchedRecords, classifications: &ClassificationMap) -> Result<Vec<Classification>, Error> {
    info!(total=records.len(), "Extracting classifications");

    let mut rows = Vec::new();
    for (dataset, record) in records {
        let id = Uuid::new_v4();

        // the classification map can be used with the scientific_name, canonical_name, or the
        // taxon_id (parent_taxon in our case). this allows us to link to the parent taxon
        // within the database and validate its correctness in the process.
        let parent_id = match record.parent_taxon {
            Some(parent) => {
                let classification = classifications
                    .get(&parent)
                    .ok_or_else(|| ParseError::NotFound(parent))?;

                classification.id.clone()
            },
            None => id.clone(),
        };

        rows.push(Classification {
            id,
            parent_id,
            dataset_id: dataset.id.clone(),
            taxon_id: record.taxon_id,
            rank: str_to_taxonomic_rank(&record.taxon_rank)?,
            accepted_name_usage: record.accepted_name_usage,
            original_name_usage: record.original_name_usage,
            scientific_name: record.scientific_name,
            scientific_name_authorship: record.scientific_name_authorship,
            canonical_name: record.canonical_name,
            nomenclatural_code: record.nomenclatural_code,
            status: str_to_taxonomic_status(&record.taxonomic_status)?,
            citation: record.citation,
            vernacular_names: record.vernacular_name.map(str_to_array),
            alternative_names: record.alternative_names.map(str_to_array),
            description: record.description,
            remarks: record.remarks,
        })
    }

    info!(rows=rows.len(), "Extracting classifications finished");
    Ok(rows)
}


fn str_to_taxonomic_rank(value: &str) -> Result<TaxonomicRank, Error> {
    match value.to_lowercase().as_str() {
        "domain" => Ok(TaxonomicRank::Domain),
        "superkingdom" => Ok(TaxonomicRank::Superkingdom),
        "kingdom" => Ok(TaxonomicRank::Kingdom),
        "subkingdom" => Ok(TaxonomicRank::Subkingdom),
        "phylum" => Ok(TaxonomicRank::Phylum),
        "subphylum" => Ok(TaxonomicRank::Subphylum),
        "superclass" => Ok(TaxonomicRank::Superclass),
        "class" => Ok(TaxonomicRank::Class),
        "subclass" => Ok(TaxonomicRank::Subclass),
        "superorder" => Ok(TaxonomicRank::Superorder),
        "order" => Ok(TaxonomicRank::Order),
        "suborder" => Ok(TaxonomicRank::Suborder),
        "superfamily" => Ok(TaxonomicRank::Superfamily),
        "family" => Ok(TaxonomicRank::Family),
        "subfamily" => Ok(TaxonomicRank::Subfamily),
        "supertribe" => Ok(TaxonomicRank::Supertribe),
        "tribe" => Ok(TaxonomicRank::Tribe),
        "subtribe" => Ok(TaxonomicRank::Subtribe),
        "genus" => Ok(TaxonomicRank::Genus),
        "subgenus" => Ok(TaxonomicRank::Subgenus),
        "species" => Ok(TaxonomicRank::Species),
        "subspecies" => Ok(TaxonomicRank::Subspecies),
        "unranked" => Ok(TaxonomicRank::Unranked),
        "higher taxon" => Ok(TaxonomicRank::HigherTaxon),

        val => Err(Error::Parsing(ParseError::InvalidValue(val.to_string()))),
    }
}

// based on https://rs.gbif.org/vocabulary/gbif/taxonomic_status.xml
fn str_to_taxonomic_status(value: &str) -> Result<TaxonomicStatus, Error> {
    match value.to_lowercase().as_str() {
        "valid" => Ok(TaxonomicStatus::Accepted),
        "valid name" => Ok(TaxonomicStatus::Accepted),
        "accepted" => Ok(TaxonomicStatus::Accepted),
        "accepted name" => Ok(TaxonomicStatus::Accepted),

        "undescribed" => Ok(TaxonomicStatus::Undescribed),
        "species inquirenda" => Ok(TaxonomicStatus::SpeciesInquirenda),
        "manuscript name" => Ok(TaxonomicStatus::ManuscriptName),
        "hybrid" => Ok(TaxonomicStatus::Hybrid),

        "synonym" => Ok(TaxonomicStatus::Synonym),
        "junior synonym" => Ok(TaxonomicStatus::Synonym),
        "later synonym" => Ok(TaxonomicStatus::Synonym),

        "invalid" => Ok(TaxonomicStatus::Unaccepted),
        "invalid name" => Ok(TaxonomicStatus::Unaccepted),
        "unaccepted" => Ok(TaxonomicStatus::Unaccepted),
        "unaccepted name" => Ok(TaxonomicStatus::Unaccepted),

        "informal" => Ok(TaxonomicStatus::Informal),

        val => Err(Error::Parsing(ParseError::InvalidValue(val.to_string()))),
    }
}

fn str_to_array(value: String) -> Vec<Option<String>> {
    value.split("|").map(|v| Some(String::from(v))).collect()
}
