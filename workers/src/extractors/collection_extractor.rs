use std::path::PathBuf;

use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Specimen, CollectionEvent, Dataset};
use crate::error::Error;
use crate::extractors::utils::parse_lat_lng;
use crate::matchers::name_matcher::{NameMatch, NameRecord, match_records_mapped, NameMap, name_map};
use crate::matchers::specimen_matcher::{SpecimenMap, specimen_map};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,
    scientific_name: Option<String>,
    canonical_name: Option<String>,

    type_status: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    collection_code: Option<String>,
    catalog_number: Option<String>,
    collected_by: Option<String>,
    identified_by: Option<String>,
    identified_date: Option<String>,
    organism_id: Option<String>,
    material_sample_id: Option<String>,
    details: Option<String>,
    remarks: Option<String>,
    identification_remarks: Option<String>,

    // location block
    locality: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    state_province: Option<String>,
    county: Option<String>,
    municipality: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    verbatim_lat_long: Option<String>,
    elevation: Option<f64>,
    depth: Option<f64>,
    elevation_accuracy: Option<f64>,
    depth_accuracy: Option<f64>,
    location_source: Option<String>,

    // collection event block
    event_date: Option<String>,
    event_time: Option<String>,
    field_number: Option<String>,
    field_notes: Option<String>,
    record_number: Option<String>,
    individual_count: Option<String>,
    organism_quantity: Option<String>,
    organism_quantity_type: Option<String>,
    sex: Option<String>,
    genotypic_sex: Option<String>,
    phenotypic_sex: Option<String>,
    life_stage: Option<String>,
    reproductive_condition: Option<String>,
    behavior: Option<String>,
    establishment_means: Option<String>,
    degree_of_establishment: Option<String>,
    pathway: Option<String>,
    occurrence_status: Option<String>,
    preparation: Option<String>,
    other_catalog_numbers: Option<String>,
    env_broad_scale: Option<String>,
    env_local_scale: Option<String>,
    env_medium: Option<String>,
    habitat: Option<String>,
    ref_biomaterial: Option<String>,
    source_mat_id: Option<String>,
    specific_host: Option<String>,
    strain: Option<String>,
    isolate: Option<String>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}


pub struct CollectionExtract {
    pub specimens: Vec<Specimen>,
    pub collection_events: Vec<CollectionEvent>,
}


pub struct CollectionExtractIterator {
    dataset: Dataset,
    names: NameMap,
    specimens: SpecimenMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for CollectionExtractIterator {
    type Item = Result<CollectionExtract, Error>;

    /// Return a large chunk of collection events extracted from a CSV reader
    fn next(&mut self) -> Option<Self::Item> {
        info!("Deserialising CSV");
        let mut records: Vec<Record> = Vec::with_capacity(1_000_000);

        // take the next million records and return early with an error result
        // if parsing failed
        for row in self.reader.by_ref().take(1_000_000) {
            match row {
                Ok(record) => records.push(record),
                Err(err) => return Some(Err(err.into()))
            }
        }

        info!(total=records.len(), "Deserialising CSV finished");

        // if empth we've reached the end, otherwise do the expensive work
        // of extracting the chunk of data within the iterator call
        if records.is_empty() {
            None
        } else {
            Some(extract_chunk(records, &self.dataset, &self.names, &self.specimens))
        }
    }
}


/// Extract collection events and other related data from a CSV file
///
/// Every collection event by it's very action must have a specimen associated with
/// it and a parent event tracking common event metadata. A specimen can be further
/// used by other events but a collection event will *always* create a new specimen
/// since it is the _collection_ of a particular specimen that it describes.
pub fn extract(path: PathBuf, dataset: &Dataset, context: &Vec<Dataset>, pool: &mut PgPool) -> Result<CollectionExtractIterator, Error> {
    let isolated_datasets = context.iter().map(|d| d.id.clone()).collect();

    let names = name_map(pool)?;
    let specimens = specimen_map(&isolated_datasets, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(CollectionExtractIterator {
        dataset: dataset.clone(),
        names,
        specimens,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, names: &NameMap, existing: &SpecimenMap) -> Result<CollectionExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, names)?;

    // extract all the records associated with a collection event.
    // these extraction methods return results in the same order as the input records
    // which makes it possible to zip the various extractions to get any associated ids
    // if necessary
    let specimens = extract_specimens(dataset, &records);
    let collection_events = extract_collection_events(dataset, records, &specimens);

    // exclude any records that already exist within the isolation context. we want to
    // allow for duplicate record ids from different datasets so we cannot leverage unique
    // index constraints at the database level
    let mut extract = CollectionExtract {
        specimens: Vec::new(),
        collection_events: Vec::new(),
    };

    for (specimen, collection_event) in specimens.into_iter().zip(collection_events.into_iter()) {
        if !existing.contains_key(&specimen.record_id) {
            extract.specimens.push(specimen);
            extract.collection_events.push(collection_event);
        }
    }

    Ok(extract)
}


fn extract_specimens(dataset: &Dataset, records: &MatchedRecords) -> Vec<Specimen> {
    info!(total=records.len(), "Extracting specimens");

    let specimens = records.par_iter().map(|(name, row)| {
        let coords = match &row.verbatim_lat_long {
            Some(lat_long) => parse_lat_lng(&lat_long).ok(),
            None => None,
        };

        Specimen {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            name_id: name.id.clone(),

            record_id: row.record_id.clone(),
            material_sample_id: row.material_sample_id.clone(),
            organism_id: row.organism_id.clone(),

            institution_name: row.institution_name.clone(),
            institution_code: row.institution_code.clone(),
            collection_code: row.collection_code.clone(),
            recorded_by: row.collected_by.clone(),
            identified_by: row.identified_by.clone(),
            identified_date: row.identified_date.clone(),

            type_status: row.type_status.clone(),
            locality: row.locality.clone(),
            latitude: row.latitude.or_else(|| coords.clone().map(|c| c.latitude)),
            longitude: row.longitude.or_else(|| coords.clone().map(|c| c.longitude)),
            country: row.country.clone(),
            country_code: row.country_code.clone(),
            state_province: row.state_province.clone(),
            county: row.county.clone(),
            municipality: row.municipality.clone(),
            elevation: row.elevation.clone(),
            depth: row.depth.clone(),
            elevation_accuracy: row.elevation_accuracy.clone(),
            depth_accuracy: row.depth_accuracy.clone(),
            location_source: row.location_source.clone(),

            details: row.details.clone(),
            remarks: row.remarks.clone(),
            identification_remarks: row.identification_remarks.clone(),
        }
    }).collect::<Vec<Specimen>>();

    info!(specimens=specimens.len(), "Extracting specimens finished");
    specimens
}


fn extract_collection_events(dataset: &Dataset, records: MatchedRecords, specimens: &Vec<Specimen>) -> Vec<CollectionEvent> {
    info!(total=records.len(), "Extracting collection events");

    let collections = (records, specimens).into_par_iter().map(|(record, specimen)| {
        let (_name, row) = record;

        CollectionEvent {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            specimen_id: specimen.id.clone(),

            event_date: row.event_date,
            event_time: row.event_time,
            collected_by: row.collected_by,

            field_number: row.field_number,
            catalog_number: row.catalog_number,
            record_number: row.record_number,
            individual_count: row.individual_count,
            organism_quantity: row.organism_quantity,
            organism_quantity_type: row.organism_quantity_type,
            sex: row.sex,
            genotypic_sex: row.genotypic_sex,
            phenotypic_sex: row.phenotypic_sex,
            life_stage: row.life_stage,
            reproductive_condition: row.reproductive_condition,
            behavior: row.behavior,
            establishment_means: row.establishment_means,
            degree_of_establishment: row.degree_of_establishment,
            pathway: row.pathway,
            occurrence_status: row.occurrence_status,
            preparation: row.preparation,
            other_catalog_numbers: row.other_catalog_numbers,

            env_broad_scale: row.env_broad_scale,
            env_local_scale: row.env_local_scale,
            env_medium: row.env_medium,
            habitat: row.habitat,
            ref_biomaterial: row.ref_biomaterial,
            source_mat_id: row.source_mat_id,
            specific_host: row.specific_host,
            strain: row.strain,
            isolate: row.isolate,

            field_notes: row.field_notes,
            remarks: row.remarks,
        }
    }).collect::<Vec<CollectionEvent>>();

    info!(collection_events=collections.len(), "Extracting collection events finished");
    collections
}
