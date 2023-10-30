use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Taxon, TaxonomicStatus, Dataset};
use crate::error::Error;
use crate::extractors::utils::{extract_authority, decompose_scientific_name};
use crate::matchers::name_matcher::{match_records, NameRecord, NameMatch};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    scientific_name: String,
    canonical_name: Option<String>,
    // rank: Option<String>,
    species_authority: Option<String>,

    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    tribe: Option<String>,
    genus: Option<String>,

    superclass: Option<String>,
    superorder: Option<String>,
    superfamily: Option<String>,
    supertribe: Option<String>,

    subphylum: Option<String>,
    subclass: Option<String>,
    suborder: Option<String>,
    subfamily: Option<String>,
    subtribe: Option<String>,
    subgenus: Option<String>,
    // subspecies: Option<String>,

    // basionym_genus: Option<String>,
    // basionym_subgenus: Option<String>,
    // basionym_species: Option<String>,
    // basionym_subspecies: Option<String>,
    // basionym_canonical_name: Option<String>,
    // basionym_author: Option<String>,
    // basionym_year: Option<String>,

    specific_epithet: Option<String>,
    subspecific_epithet: Option<String>,

    species: Option<String>,
    genus_full: Option<String>,
    family_full: Option<String>,
    order_full: Option<String>,

    // name_according_to: Option<String>,
    // name_published_in: Option<String>,

    taxonomic_status: Option<String>,
    // taxon_remarks: Option<String>,
    // source: Option<String>,
    // source_url: Option<String>,
    // source_id: Option<String>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: Some(value.scientific_name),
            canonical_name: value.canonical_name,
        }
    }
}


/// Extract names and taxonomy from a CSV file
pub fn extract(path: &PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<Vec<Taxon>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(records, pool);
    let taxa = extract_taxa(&dataset, &records);
    Ok(taxa)
}


fn extract_taxa(dataset: &Dataset, records: &MatchedRecords) -> Vec<Taxon> {
    info!(total=records.len(), "Extracting taxa");

    let taxa = records.par_iter().map(|(name, row)| {
        let order_authority = extract_authority(&row.order, &row.order_full);
        let family_authority = extract_authority(&row.family, &row.family_full);
        let genus_authority = extract_authority(&row.genus, &row.genus_full);

        // if certain fields making up a scientific name can't be found try
        // to extract it from the scientific name
        let decomposed = decompose_scientific_name(&row.scientific_name);

        let genus = match &row.genus {
            Some(genus) => Some(genus.clone()),
            None => decomposed.clone().map(|v| v.genus),
        };

        let subgenus = match &row.subgenus {
            Some(subgenus) => Some(subgenus.clone()),
            None => decomposed.clone().and_then(|v| v.subgenus),
        };

        let specific_epithet = match &row.specific_epithet {
            Some(specific_epithet) => Some(specific_epithet.clone()),
            None => decomposed.clone().map(|v| v.specific_epithet),
        };

        let subspecific_epithet = match &row.subspecific_epithet {
            Some(subspecific_epithet) => Some(subspecific_epithet.clone()),
            None => decomposed.clone().and_then(|v| v.subspecific_epithet),
        };

        let species_authority = match &row.species_authority {
            Some(authority) => Some(authority.clone()),
            None => match &row.species {
                Some(_) => extract_authority(&row.canonical_name, &row.species),
                None => decomposed.clone().map(|v| v.authority),
            },
        };

        let canonical_name = match &row.canonical_name {
            Some(canonical_name) => Some(canonical_name.clone()),
            None => decomposed.map(|v| v.canonical_name())
        };

        Taxon {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            name_id: name.id.clone(),

            status: str_to_taxonomic_status(&row.taxonomic_status),
            scientific_name: row.scientific_name.clone(),
            canonical_name: canonical_name.unwrap_or_else(|| row.scientific_name.clone()),

            kingdom: row.kingdom.clone(),
            phylum: row.phylum.clone(),
            class: row.class.clone(),
            order: row.order.clone(),
            family: row.family.clone(),
            tribe: row.tribe.clone(),
            genus,
            specific_epithet,

            subphylum: row.subphylum.clone(),
            subclass: row.subclass.clone(),
            suborder: row.suborder.clone(),
            subfamily: row.subfamily.clone(),
            subtribe: row.subtribe.clone(),
            subgenus,
            subspecific_epithet,

            superclass: row.superclass.clone(),
            superorder: row.superorder.clone(),
            superfamily: row.superfamily.clone(),
            supertribe: row.supertribe.clone(),

            order_authority,
            family_authority,
            genus_authority,
            species_authority,
        }
    }).collect::<Vec<Taxon>>();

    info!(taxa=taxa.len(), "Extracting taxa finished");
    taxa
}


// based on https://rs.gbif.org/vocabulary/gbif/taxonomic_status.xml
fn str_to_taxonomic_status(value: &Option<String>) -> TaxonomicStatus {
    match value {
        Some(status) => match status.to_lowercase().as_str() {
            "valid" => TaxonomicStatus::Accepted,
            "valid name" => TaxonomicStatus::Accepted,
            "accepted" => TaxonomicStatus::Accepted,
            "accepted name" => TaxonomicStatus::Accepted,

            "undescribed" => TaxonomicStatus::Undescribed,
            "species inquirenda" => TaxonomicStatus::SpeciesInquirenda,
            "manuscript name" => TaxonomicStatus::ManuscriptName,
            "hybrid" => TaxonomicStatus::Hybrid,

            "synonym" => TaxonomicStatus::Synonym,
            "junior synonym" => TaxonomicStatus::Synonym,
            "later synonym" => TaxonomicStatus::Synonym,

            "invalid" => TaxonomicStatus::Unaccepted,
            "invalid name" => TaxonomicStatus::Unaccepted,
            "unaccepted" => TaxonomicStatus::Unaccepted,
            "unaccepted name" => TaxonomicStatus::Unaccepted,

            _ => TaxonomicStatus::Unaccepted,
        },
        None => TaxonomicStatus::Unaccepted,
    }
}
