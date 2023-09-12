use std::path::PathBuf;

use chrono::NaiveDate;
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Event, Dataset, AccessionEvent};
use crate::error::Error;
use crate::matchers::name_matcher::{NameMatch, NameRecord, match_records_mapped, NameMap, name_map};

use super::utils::naive_date_from_str_opt;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    type_status: Option<String>,
    accession: Option<String>,

    #[serde(rename(deserialize = "materialSampleID"))]
    material_sample_id: Option<String>,

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
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}


pub struct AccessionExtract {
    pub events: Vec<Event>,
    pub accession_events: Vec<AccessionEvent>,
}


pub struct AccessionExtractIterator {
    pool: PgPool,
    dataset: Dataset,
    names: NameMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for AccessionExtractIterator {
    type Item = Result<AccessionExtract, Error>;

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
            Some(extract_chunk(records, &self.dataset, &self.names, &mut self.pool))
        }
    }
}


/// Extract accession events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<AccessionExtractIterator, Error> {
    let names = name_map(pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(AccessionExtractIterator {
        pool: pool.clone(),
        dataset: dataset.clone(),
        names,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, names: &NameMap, pool: &mut PgPool) -> Result<AccessionExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, names)?;

    let events = extract_events(&records);
    let accession_events = extract_accession_events(&records, dataset, &events);

    Ok(AccessionExtract {
        events,
        accession_events,
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


fn extract_accession_events(records: &MatchedRecords, dataset: &Dataset, events: &Vec<Event>) -> Vec<AccessionEvent>
{
    info!(total=records.len(), "Extracting accession events");

    let accessions = (records, events).into_par_iter().map(|(record, event)| {
        let (name, row) = record;

        AccessionEvent {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            name_id: name.id.clone(),
            event_id: event.id.clone(),

            accession: row.accession.clone(),
            material_sample_id: row.material_sample_id.clone(),
            institution_name: row.institution_name.clone(),
            institution_code: row.institution_code.clone(),
            type_status: row.type_status.clone(),
        }
    }).collect::<Vec<AccessionEvent>>();

    info!(accession_events=accessions.len(), "Extracting accession events finished");
    accessions
}
