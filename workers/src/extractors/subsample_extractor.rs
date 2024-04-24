use std::path::PathBuf;

use arga_core::models::{Dataset, Subsample, SubsampleEvent};
use csv::DeserializeRecordsIntoIter;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;
use crate::matchers::specimen_matcher::{
    match_records_mapped,
    specimen_map,
    SpecimenMap,
    SpecimenMatch,
    SpecimenRecord,
};
use crate::matchers::subsample_matcher::{subsample_map, SubsampleMap};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SpecimenMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,
    specimen_id: String,
    entity_id: Option<String>,
    preparation_type: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,
    subsampled_by: Option<String>,

    // dna block
    material_sample_id: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    type_status: Option<String>,
}

impl From<Record> for SpecimenRecord {
    fn from(value: Record) -> Self {
        Self {
            record_id: value.specimen_id,
        }
    }
}


pub struct SubsampleExtract {
    pub subsamples: Vec<Subsample>,
    pub subsample_events: Vec<SubsampleEvent>,
}


pub struct SubsampleExtractIterator {
    dataset: Dataset,
    specimens: SpecimenMap,
    subsamples: SubsampleMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for SubsampleExtractIterator {
    type Item = Result<SubsampleExtract, Error>;

    /// Return a large chunk of events extracted from a CSV reader
    fn next(&mut self) -> Option<Self::Item> {
        info!("Deserialising CSV");
        let mut records: Vec<Record> = Vec::with_capacity(1_000_000);

        // take the next million records and return early with an error result
        // if parsing failed
        for row in self.reader.by_ref().take(1_000_000) {
            match row {
                Ok(record) => records.push(record),
                Err(err) => return Some(Err(err.into())),
            }
        }

        info!(total = records.len(), "Deserialising CSV finished");

        // if empth we've reached the end, otherwise do the expensive work
        // of extracting the chunk of data within the iterator call
        if records.is_empty() {
            None
        }
        else {
            Some(extract_chunk(records, &self.dataset, &self.specimens, &self.subsamples))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(
    path: PathBuf,
    dataset: &Dataset,
    context: &Vec<Dataset>,
    pool: &mut PgPool,
) -> Result<SubsampleExtractIterator, Error> {
    let isolated_datasets = context.iter().map(|d| d.id.clone()).collect();

    let specimens = specimen_map(&isolated_datasets, pool)?;
    let subsamples = subsample_map(&isolated_datasets, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(SubsampleExtractIterator {
        dataset: dataset.clone(),
        specimens,
        subsamples,
        reader,
    })
}


fn extract_chunk(
    chunk: Vec<Record>,
    dataset: &Dataset,
    specimens: &SpecimenMap,
    existing: &SubsampleMap,
) -> Result<SubsampleExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, specimens);

    let subsamples = extract_subsamples(dataset, &records);
    let subsample_events = extract_subsample_events(dataset, records, &subsamples);

    // exclude any records that already exist within the isolation context. we want to
    // allow for duplicate record ids from different datasets so we cannot leverage unique
    // index constraints at the database level
    let mut extract = SubsampleExtract {
        subsamples: Vec::new(),
        subsample_events: Vec::new(),
    };

    for (subsample, subsample_event) in subsamples.into_iter().zip(subsample_events.into_iter()) {
        if !existing.contains_key(&subsample.record_id) {
            extract.subsamples.push(subsample);
            extract.subsample_events.push(subsample_event);
        }
    }

    Ok(extract)
}


fn extract_subsamples(dataset: &Dataset, records: &MatchedRecords) -> Vec<Subsample> {
    info!(total = records.len(), "Extracting subsamples");

    let subsamples = records
        .par_iter()
        .map(|(specimen, row)| Subsample {
            id: Uuid::new_v4(),
            dataset_id: dataset.id,
            name_id: specimen.name_id,
            specimen_id: specimen.id,
            entity_id: row.entity_id.clone(),

            record_id: row.record_id.clone(),
            material_sample_id: row.material_sample_id.clone(),
            institution_name: row.institution_name.clone(),
            institution_code: row.institution_code.clone(),
            type_status: row.type_status.clone(),
        })
        .collect::<Vec<Subsample>>();

    info!(events = subsamples.len(), "Extracting subsamples finished");
    subsamples
}


fn extract_subsample_events(
    dataset: &Dataset,
    records: MatchedRecords,
    subsamples: &Vec<Subsample>,
) -> Vec<SubsampleEvent> {
    info!(total = records.len(), "Extracting subsample events");

    let subsample_events = (records, subsamples)
        .into_par_iter()
        .map(|(record, subsample)| {
            let (_specimen, row) = record;

            SubsampleEvent {
                id: Uuid::new_v4(),
                dataset_id: dataset.id.clone(),
                subsample_id: subsample.id.clone(),
                entity_id: row.entity_id,
                event_date: row.event_date,
                event_time: row.event_time,
                subsampled_by: row.subsampled_by,
                preparation_type: row.preparation_type,
            }
        })
        .collect::<Vec<SubsampleEvent>>();

    info!(subsample_events = subsample_events.len(), "Extracting subsample events finished");
    subsample_events
}
