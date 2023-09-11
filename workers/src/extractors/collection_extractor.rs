use std::path::PathBuf;

use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Specimen, Event, CollectionEvent, Organism, Dataset};
use crate::error::Error;
use crate::extractors::utils::parse_lat_lng;
use crate::matchers::name_matcher::{match_records, NameMatch, NameRecord};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    canonical_name: Option<String>,
    type_status: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    collection_code: Option<String>,
    catalog_number: Option<String>,
    recorded_by: Option<String>,
    #[serde(rename(deserialize = "organismID"))]
    organism_id: Option<String>,
    locality: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    verbatim_lat_long: Option<String>,
    details: Option<String>,
    remarks: Option<String>,

    // event block
    #[serde(rename(deserialize = "eventID"))]
    event_id: Option<String>,
    // #[serde(rename(deserialize = "parentEventID"))]
    // parent_event_id: Option<String>,
    field_number: Option<String>,
    event_date: Option<chrono::DateTime<chrono::Utc>>,
    habitat: Option<String>,
    sampling_protocol: Option<String>,
    sampling_size_value: Option<String>,
    sampling_size_unit: Option<String>,
    sampling_effort: Option<String>,
    field_notes: Option<String>,
    event_remarks: Option<String>,

    // collection event block
    accession: Option<String>,
    record_number: Option<String>,
    individual_count: Option<String>,
    organism_quantity: Option<String>,
    organism_quantity_type: Option<String>,
    sex: Option<String>,
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
    ref_biomaterial: Option<String>,
    source_mat_id: Option<String>,
    specific_host: Option<String>,
    strain: Option<String>,
    isolate: Option<String>,

    // organism block
    organism_name: Option<String>,
    organism_scope: Option<String>,
    associated_organisms: Option<String>,
    previous_identifications: Option<String>,
    organism_remarks: Option<String>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: Some(value.scientific_name),
            canonical_name: value.canonical_name,
        }
    }
}


pub struct CollectionExtract {
    pub specimens: Vec<Specimen>,
    pub organisms: Vec<Organism>,
    pub events: Vec<Event>,
    pub collection_events: Vec<CollectionEvent>,
}


pub struct CollectionExtractIterator {
    pool: PgPool,
    dataset: Dataset,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for CollectionExtractIterator {
    type Item = Result<CollectionExtract, Error>;

    /// Return a large chunk of collection events extracted from a CSV reader
    fn next(&mut self) -> Option<Self::Item> {
        let mut records: Vec<Record> = Vec::with_capacity(1_000_000);

        // take the next million records and return early with an error result
        // if parsing failed
        for row in self.reader.by_ref().take(1_000_000) {
            match row {
                Ok(record) => records.push(record),
                Err(err) => return Some(Err(err.into()))
            }
        }

        // if empth we've reached the end, otherwise do the expensive work
        // of extracting the chunk of data within the iterator call
        if records.is_empty() {
            None
        } else {
            Some(extract_chunk(records, &self.dataset, &mut self.pool))
        }
    }
}


/// Extract collection events and other related data from a CSV file
///
/// Every collection event by it's very action must have a specimen associated with
/// it and a parent event tracking common event metadata. A specimen can be further
/// used by other events but a collection event will *always* create a new specimen
/// since it is the _collection_ of a particular specimen that it describes.
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<CollectionExtractIterator, Error> {
    let reader = csv::Reader::from_path(&path)?.into_deserialize();
    Ok(CollectionExtractIterator {
        pool: pool.clone(),
        dataset: dataset.clone(),
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, pool: &mut PgPool) -> Result<CollectionExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(chunk, pool);

    // extract all the records associated with a collection event.
    // these extraction method return results in the same order as the input records
    // which makes it possible to zip the various extractions to get any associated ids
    // if necessary
    let specimens = extract_specimens(dataset, &records);
    let organisms = extract_organisms(&records);
    let events = extract_events(&records);

    let collection_events = extract_collection_events(&records, &specimens, &events, &organisms);
    let organisms = organisms.into_iter().filter_map(|o| o).collect::<Vec<Organism>>();

    Ok(CollectionExtract {
        specimens,
        organisms,
        events,
        collection_events,
    })
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
            type_status: row.type_status.clone(),
            institution_name: row.institution_name.clone(),
            institution_code: row.institution_code.clone(),
            collection_code: row.collection_code.clone(),
            catalog_number: row.catalog_number.clone(),
            recorded_by: row.recorded_by.clone(),
            organism_id: row.organism_id.clone(),
            locality: row.locality.clone(),
            latitude: row.latitude.or_else(|| coords.clone().map(|c| c.latitude)),
            longitude: row.longitude.or_else(|| coords.clone().map(|c| c.longitude)),
            details: row.details.clone(),
            remarks: row.remarks.clone(),
        }
    }).collect::<Vec<Specimen>>();

    info!(specimens=specimens.len(), "Extracting specimens finished");
    specimens
}


fn extract_organisms(records: &MatchedRecords) -> Vec<Option<Organism>> {
    info!(total=records.len(), "Extracting organisms");

    let organisms = records.par_iter().map(|(name, row)| {
        match &row.organism_id {
            Some(organism_id) => Some(Organism {
                id: Uuid::new_v4(),
                name_id: name.id.clone(),
                organism_id: Some(organism_id.clone()),
                organism_name: row.organism_name.clone(),
                organism_scope: row.organism_scope.clone(),
                associated_organisms: row.associated_organisms.clone(),
                previous_identifications: row.previous_identifications.clone(),
                remarks: row.organism_remarks.clone(),
            }),
            _ => None,
        }
    }).collect::<Vec<Option<Organism>>>();

    info!(organisms=organisms.len(), "Extracting organisms finished");
    organisms
}


fn extract_events(records: &MatchedRecords) -> Vec<Event> {
    info!(total=records.len(), "Extracting events");

    let events = records.par_iter().map(|(_name, row)| {
        Event {
            id: Uuid::new_v4(),
            parent_event_id: None,
            event_id: row.event_id.clone(),
            field_number: row.field_number.clone(),
            event_date: row.event_date.map(|d| d.date_naive()).clone(),
            habitat: row.habitat.clone(),
            sampling_protocol: row.sampling_protocol.clone(),
            sampling_size_value: row.sampling_size_value.clone(),
            sampling_size_unit: row.sampling_size_unit.clone(),
            sampling_effort: row.sampling_effort.clone(),
            field_notes: row.field_notes.clone(),
            event_remarks: row.event_remarks.clone(),
        }
    }).collect::<Vec<Event>>();

    info!(events=events.len(), "Extracting events finished");
    events
}


fn extract_collection_events(
    records: &MatchedRecords,
    specimens: &Vec<Specimen>,
    events: &Vec<Event>,
    organisms: &Vec<Option<Organism>>,
) -> Vec<CollectionEvent>
{
    info!(total=records.len(), "Extracting collection events");

    let collections = (records, specimens, events, organisms).into_par_iter().map(|(record, specimen, event, organism)| {
        let (_name, row) = record;

        CollectionEvent {
            id: Uuid::new_v4(),
            event_id: event.id.clone(),
            specimen_id: specimen.id.clone(),
            organism_id: organism.clone().map(|o| o.id),

            accession: row.accession.clone(),
            catalog_number: row.catalog_number.clone(),
            record_number: row.record_number.clone(),
            individual_count: row.individual_count.clone(),
            organism_quantity: row.organism_quantity.clone(),
            organism_quantity_type: row.organism_quantity_type.clone(),
            sex: row.sex.clone(),
            life_stage: row.life_stage.clone(),
            reproductive_condition: row.reproductive_condition.clone(),
            behavior: row.behavior.clone(),
            establishment_means: row.establishment_means.clone(),
            degree_of_establishment: row.degree_of_establishment.clone(),
            pathway: row.pathway.clone(),
            occurrence_status: row.occurrence_status.clone(),
            preparation: row.preparation.clone(),
            other_catalog_numbers: row.other_catalog_numbers.clone(),
            env_broad_scale: row.env_broad_scale.clone(),
            ref_biomaterial: row.ref_biomaterial.clone(),
            source_mat_id: row.source_mat_id.clone(),
            specific_host: row.specific_host.clone(),
            strain: row.strain.clone(),
            isolate: row.isolate.clone(),
        }
    }).collect::<Vec<CollectionEvent>>();

    info!(collection_events=collections.len(), "Extracting collection events finished");
    collections
}
