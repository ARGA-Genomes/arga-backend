use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::models::{Taxon, TaxonomicRank, TaxonomicStatus};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::{Error, ParseError};
use crate::matchers::classification_matcher::{classification_map, classification_map_scoped, ClassificationMap};
use crate::matchers::dataset_matcher::{dataset_map, match_records, DatasetMatch, DatasetRecord};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(DatasetMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    parent_taxon: Option<String>,
    parent_scientific_name: Option<String>,
    parent_rank: Option<String>,

    taxon_id: Option<String>,
    taxon_rank: String,
    dataset_id: String,
    accepted_name_usage: Option<String>,
    original_name_usage: Option<String>,
    scientific_name: String,
    scientific_name_authorship: Option<String>,
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
            // taxon_id: record.taxon_id.map(parse_taxon_id).unwrap_or(None),
            rank: str_to_taxonomic_rank(&record.taxon_rank)?,
            // accepted_name_usage: record.accepted_name_usage,
            // original_name_usage: record.original_name_usage,
            scientific_name: record.scientific_name,
            authorship: record.scientific_name_authorship,
            canonical_name: record.canonical_name,
            nomenclatural_code: record.nomenclatural_code,
            status: str_to_taxonomic_status(&record.taxonomic_status)?,
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


fn str_to_taxonomic_rank(value: &str) -> Result<TaxonomicRank, Error> {
    match value.to_lowercase().as_str() {
        "domain" => Ok(TaxonomicRank::Domain),
        "superkingdom" => Ok(TaxonomicRank::Superkingdom),
        "kingdom" => Ok(TaxonomicRank::Kingdom),
        "subkingdom" => Ok(TaxonomicRank::Subkingdom),
        "infrakingdom" => Ok(TaxonomicRank::Infrakingdom),
        "superphylum" => Ok(TaxonomicRank::Superphylum),
        "phylum" => Ok(TaxonomicRank::Phylum),
        "subphylum" => Ok(TaxonomicRank::Subphylum),
        "infraphylum" => Ok(TaxonomicRank::Infraphylum),
        "parvphylum" => Ok(TaxonomicRank::Parvphylum),
        "gigaclass" => Ok(TaxonomicRank::Gigaclass),
        "megaclass" => Ok(TaxonomicRank::Megaclass),
        "superclass" => Ok(TaxonomicRank::Superclass),
        "class" => Ok(TaxonomicRank::Class),
        "subclass" => Ok(TaxonomicRank::Subclass),
        "infraclass" => Ok(TaxonomicRank::Infraclass),
        "subterclass" => Ok(TaxonomicRank::Subterclass),
        "superorder" => Ok(TaxonomicRank::Superorder),
        "order" => Ok(TaxonomicRank::Order),
        "hyporder" => Ok(TaxonomicRank::Hyporder),
        "minorder" => Ok(TaxonomicRank::Minorder),
        "suborder" => Ok(TaxonomicRank::Suborder),
        "infraorder" => Ok(TaxonomicRank::Infraorder),
        "parvorder" => Ok(TaxonomicRank::Parvorder),
        "epifamily" => Ok(TaxonomicRank::Epifamily),
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
        "variety" => Ok(TaxonomicRank::Variety),
        "subvariety" => Ok(TaxonomicRank::Subvariety),
        "natio" => Ok(TaxonomicRank::Natio),
        "mutatio" => Ok(TaxonomicRank::Mutatio),
        "unranked" => Ok(TaxonomicRank::Unranked),
        "higher taxon" => Ok(TaxonomicRank::HigherTaxon),
        "aggregate genera" => Ok(TaxonomicRank::AggregateGenera),
        "aggregate species" => Ok(TaxonomicRank::AggregateSpecies),
        "cohort" => Ok(TaxonomicRank::Cohort),
        "subcohort" => Ok(TaxonomicRank::Subcohort),
        "division" => Ok(TaxonomicRank::Division),
        "phylum (division)" => Ok(TaxonomicRank::Division),
        "incertae sedis" => Ok(TaxonomicRank::IncertaeSedis),
        "infragenus" => Ok(TaxonomicRank::Infragenus),
        "section" => Ok(TaxonomicRank::Section),
        "subsection" => Ok(TaxonomicRank::Subsection),
        "subdivision" => Ok(TaxonomicRank::Subdivision),
        "subphylum (subdivision)" => Ok(TaxonomicRank::Subdivision),

        "regnum" => Ok(TaxonomicRank::Regnum),
        "familia" => Ok(TaxonomicRank::Familia),
        "classis" => Ok(TaxonomicRank::Classis),
        "ordo" => Ok(TaxonomicRank::Ordo),
        "varietas" => Ok(TaxonomicRank::Varietas),
        "forma" => Ok(TaxonomicRank::Forma),
        "subforma" => Ok(TaxonomicRank::Subforma),
        "subclassis" => Ok(TaxonomicRank::Subclassis),
        "superordo" => Ok(TaxonomicRank::Superordo),
        "sectio" => Ok(TaxonomicRank::Sectio),
        "subsectio" => Ok(TaxonomicRank::Subsectio),
        "nothovarietas" => Ok(TaxonomicRank::Nothovarietas),
        "subvarietas" => Ok(TaxonomicRank::Subvarietas),
        "series" => Ok(TaxonomicRank::Series),
        "subseries" => Ok(TaxonomicRank::Subseries),
        "superspecies" => Ok(TaxonomicRank::Superspecies),
        "infraspecies" => Ok(TaxonomicRank::Infraspecies),
        "subfamilia" => Ok(TaxonomicRank::Subfamilia),
        "subordo" => Ok(TaxonomicRank::Subordo),
        "regio" => Ok(TaxonomicRank::Regio),
        "special form" => Ok(TaxonomicRank::SpecialForm),

        "" => Ok(TaxonomicRank::Unranked),

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
        "taxon inquirendum" => Ok(TaxonomicStatus::TaxonInquirendum),
        "manuscript name" => Ok(TaxonomicStatus::ManuscriptName),
        "hybrid" => Ok(TaxonomicStatus::Hybrid),

        "unassessed" => Ok(TaxonomicStatus::Unassessed),
        "unavailable name" => Ok(TaxonomicStatus::Unavailable),
        "uncertain" => Ok(TaxonomicStatus::Uncertain),
        "unjustified emendation" => Ok(TaxonomicStatus::UnjustifiedEmendation),

        "synonym" => Ok(TaxonomicStatus::Synonym),
        "junior synonym" => Ok(TaxonomicStatus::Synonym),
        "junior objective synonym" => Ok(TaxonomicStatus::Synonym),
        "junior subjective synonym" => Ok(TaxonomicStatus::Synonym),
        "later synonym" => Ok(TaxonomicStatus::Synonym),

        "homonym" => Ok(TaxonomicStatus::Homonym),
        "junior homonym" => Ok(TaxonomicStatus::Homonym),
        "unreplaced junior homonym" => Ok(TaxonomicStatus::Homonym),

        "invalid" => Ok(TaxonomicStatus::Unaccepted),
        "invalid name" => Ok(TaxonomicStatus::Unaccepted),
        "unaccepted" => Ok(TaxonomicStatus::Unaccepted),
        "unaccepted name" => Ok(TaxonomicStatus::Unaccepted),
        // "excluded" => Ok(TaxonomicStatus::Unaccepted),
        "informal" => Ok(TaxonomicStatus::Informal),
        "informal name" => Ok(TaxonomicStatus::Informal),

        "placeholder" => Ok(TaxonomicStatus::Placeholder),
        "temporary name" => Ok(TaxonomicStatus::Placeholder),

        "basionym" => Ok(TaxonomicStatus::Basionym),
        "nomenclatural synonym" => Ok(TaxonomicStatus::NomenclaturalSynonym),
        "taxonomic synonym" => Ok(TaxonomicStatus::TaxonomicSynonym),
        "replaced synonym" => Ok(TaxonomicStatus::ReplacedSynonym),

        "incorrect original spelling" => Ok(TaxonomicStatus::Misspelled),
        "misspelling" => Ok(TaxonomicStatus::Misspelled),

        "orthographic variant" => Ok(TaxonomicStatus::OrthographicVariant),
        "excluded" => Ok(TaxonomicStatus::Excluded),

        "misapplied" => Ok(TaxonomicStatus::Misapplied),
        "misapplication" => Ok(TaxonomicStatus::Misapplied),
        "alternative name" => Ok(TaxonomicStatus::AlternativeName),
        "alternative representation" => Ok(TaxonomicStatus::AlternativeName),

        "pro parte misapplied" => Ok(TaxonomicStatus::ProParteMisapplied),
        "pro parte taxonomic synonym" => Ok(TaxonomicStatus::ProParteTaxonomicSynonym),

        "doubtful misapplied" => Ok(TaxonomicStatus::DoubtfulMisapplied),
        "doubtful taxonomic synonym" => Ok(TaxonomicStatus::DoubtfulTaxonomicSynonym),
        "doubtful pro parte misapplied" => Ok(TaxonomicStatus::DoubtfulProParteMisapplied),
        "doubtful pro parte taxonomic synonym" => Ok(TaxonomicStatus::DoubtfulProParteTaxonomicSynonym),

        "nomen dubium" => Ok(TaxonomicStatus::NomenDubium),
        "nomen nudum" => Ok(TaxonomicStatus::NomenNudum),
        "nomen oblitum" => Ok(TaxonomicStatus::NomenOblitum),

        "interim unpublished" => Ok(TaxonomicStatus::InterimUnpublished),
        "superseded combination" => Ok(TaxonomicStatus::SupersededCombination),
        "superseded rank" => Ok(TaxonomicStatus::SupersededRank),
        "incorrect grammatical agreement of specific epithet" => {
            Ok(TaxonomicStatus::IncorrectGrammaticalAgreementOfSpecificEpithet)
        }

        val => Err(Error::Parsing(ParseError::InvalidValue(val.to_string()))),
    }
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
