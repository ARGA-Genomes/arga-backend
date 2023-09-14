use std::path::PathBuf;

use chrono::NaiveDate;
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Event, Dataset, SubsampleEvent, Subsample};
use crate::error::Error;
use crate::matchers::specimen_matcher::{SpecimenMatch, SpecimenRecord, SpecimenMap, specimen_map, match_records_mapped};

use super::utils::naive_date_from_str_opt;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SpecimenMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    accession: String,
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    preparation_type: Option<String>,

    // event block
    field_number: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "naive_date_from_str_opt")]
    event_date: Option<NaiveDate>,
    // event_time: Option<NaiveTime>,
    habitat: Option<String>,
    sampling_protocol: Option<String>,
    sampling_size_value: Option<String>,
    sampling_size_unit: Option<String>,
    sampling_effort: Option<String>,
    field_notes: Option<String>,
    event_remarks: Option<String>,

    // dna block
    material_sample_id: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    type_status: Option<String>,
}

impl From<Record> for SpecimenRecord {
    fn from(value: Record) -> Self {
        Self { accession: value.accession }
    }
}


pub struct SubsampleExtract {
    pub events: Vec<Event>,
    pub subsamples: Vec<Subsample>,
    pub subsample_events: Vec<SubsampleEvent>,
}


pub struct SubsampleExtractIterator {
    pool: PgPool,
    dataset: Dataset,
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
            Some(extract_chunk(records, &self.dataset, &self.specimens, &mut self.pool))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<SubsampleExtractIterator, Error> {
    let specimens = specimen_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(SubsampleExtractIterator {
        pool: pool.clone(),
        dataset: dataset.clone(),
        specimens,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, specimens: &SpecimenMap, pool: &mut PgPool) -> Result<SubsampleExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, specimens);

    let events = extract_events(&records);
    let subsamples = extract_subsamples(&records);
    let subsample_events = extract_subsample_events(&records, &subsamples, &events);

    Ok(SubsampleExtract {
        events,
        subsamples,
        subsample_events,
    })
}


fn extract_events(records: &MatchedRecords) -> Vec<Event> {
    info!(total=records.len(), "Extracting events");

    let events = records.par_iter().map(|(_name, row)| {
        Event {
            id: Uuid::new_v4(),
            field_number: row.field_number.clone(),
            event_date: row.event_date.clone(),
            event_time: None,
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


fn extract_subsamples(records: &MatchedRecords) -> Vec<Subsample> {
    info!(total=records.len(), "Extracting subsamples");

    let subsamples = records.par_iter().map(|(specimen, row)| {
        Subsample {
            id: Uuid::new_v4(),
            dataset_id: specimen.dataset_id,
            name_id: specimen.name_id,
            specimen_id: specimen.id,

            accession: row.accession.clone(),
            material_sample_id: row.material_sample_id.clone(),
            institution_name: row.institution_name.clone(),
            institution_code: row.institution_code.clone(),
            type_status: row.type_status.clone(),
        }
    }).collect::<Vec<Subsample>>();

    info!(events=subsamples.len(), "Extracting subsamples finished");
    subsamples
}


fn extract_subsample_events(records: &MatchedRecords, subsamples: &Vec<Subsample>, events: &Vec<Event>) -> Vec<SubsampleEvent>
{
    info!(total=records.len(), "Extracting subsample events");

    let subsample_events = (records, subsamples, events).into_par_iter().map(|(record, subsample, event)| {
        let (_specimen, row) = record;

        SubsampleEvent {
            id: Uuid::new_v4(),
            subsample_id: subsample.id.clone(),
            event_id: event.id.clone(),
            preparation_type: row.preparation_type.clone(),
        }
    }).collect::<Vec<SubsampleEvent>>();

    info!(subsample_events=subsample_events.len(), "Extracting subsample events finished");
    subsample_events
}
