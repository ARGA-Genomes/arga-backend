use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Dataset, SubsampleEvent, Subsample};
use crate::error::Error;
use crate::matchers::specimen_matcher::{SpecimenMatch, SpecimenRecord, SpecimenMap, specimen_map, match_records_mapped};

use super::utils::naive_date_from_str_opt;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SpecimenMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    record_id: String,
    preparation_type: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "naive_date_from_str_opt")]
    event_date: Option<NaiveDate>,
    event_time: Option<NaiveTime>,
    subsampled_by: Option<String>,

    // dna block
    material_sample_id: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    type_status: Option<String>,
}

impl From<Record> for SpecimenRecord {
    fn from(value: Record) -> Self {
        Self { record_id: value.record_id }
    }
}


pub struct SubsampleExtract {
    pub subsamples: Vec<Subsample>,
    pub subsample_events: Vec<SubsampleEvent>,
}


pub struct SubsampleExtractIterator {
    specimens: SpecimenMap,
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
                Err(err) => return Some(Err(err.into()))
            }
        }

        info!(total=records.len(), "Deserialising CSV finished");

        // if empth we've reached the end, otherwise do the expensive work
        // of extracting the chunk of data within the iterator call
        if records.is_empty() {
            None
        } else {
            Some(extract_chunk(records, &self.specimens))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<SubsampleExtractIterator, Error> {
    let specimens = specimen_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(SubsampleExtractIterator {
        specimens,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, specimens: &SpecimenMap) -> Result<SubsampleExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, specimens);

    let subsamples = extract_subsamples(&records);
    let subsample_events = extract_subsample_events(records, &subsamples);

    Ok(SubsampleExtract {
        subsamples,
        subsample_events,
    })
}


fn extract_subsamples(records: &MatchedRecords) -> Vec<Subsample> {
    info!(total=records.len(), "Extracting subsamples");

    let subsamples = records.par_iter().map(|(specimen, row)| {
        Subsample {
            id: Uuid::new_v4(),
            dataset_id: specimen.dataset_id,
            name_id: specimen.name_id,
            specimen_id: specimen.id,

            record_id: row.record_id.clone(),
            material_sample_id: row.material_sample_id.clone(),
            institution_name: row.institution_name.clone(),
            institution_code: row.institution_code.clone(),
            type_status: row.type_status.clone(),
        }
    }).collect::<Vec<Subsample>>();

    info!(events=subsamples.len(), "Extracting subsamples finished");
    subsamples
}


fn extract_subsample_events(records: MatchedRecords, subsamples: &Vec<Subsample>) -> Vec<SubsampleEvent>
{
    info!(total=records.len(), "Extracting subsample events");

    let subsample_events = (records, subsamples).into_par_iter().map(|(record, subsample)| {
        let (_specimen, row) = record;

        SubsampleEvent {
            id: Uuid::new_v4(),
            subsample_id: subsample.id.clone(),
            event_date: row.event_date,
            event_time: row.event_time,
            subsampled_by: row.subsampled_by,
            preparation_type: row.preparation_type,
        }
    }).collect::<Vec<SubsampleEvent>>();

    info!(subsample_events=subsample_events.len(), "Extracting subsample events finished");
    subsample_events
}
