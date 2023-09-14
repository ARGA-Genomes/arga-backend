use std::path::PathBuf;

use chrono::NaiveDate;
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Event, Dataset, AssemblyEvent};
use crate::error::Error;
use crate::matchers::sequence_matcher::{SequenceMatch, SequenceRecord, SequenceMap, sequence_map, match_records_mapped};

use super::utils::naive_date_from_str_opt;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SequenceMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    accession: String,
    scientific_name: Option<String>,
    canonical_name: Option<String>,

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

    // assembly block
    assembly_name: Option<String>,
    version_status: Option<String>,
    assembly_quality: Option<String>,
}

impl From<Record> for SequenceRecord {
    fn from(value: Record) -> Self {
        Self { accession: value.accession }
    }
}


pub struct AssemblyExtract {
    pub events: Vec<Event>,
    pub assembly_events: Vec<AssemblyEvent>,
}


pub struct AssemblyExtractIterator {
    dataset: Dataset,
    sequences: SequenceMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for AssemblyExtractIterator {
    type Item = Result<AssemblyExtract, Error>;

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
            Some(extract_chunk(records, &self.dataset, &self.sequences))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<AssemblyExtractIterator, Error> {
    let sequences = sequence_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(AssemblyExtractIterator {
        dataset: dataset.clone(),
        sequences,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, sequences: &SequenceMap) -> Result<AssemblyExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, sequences);

    let events = extract_events(&records);
    let assembly_events = extract_assembly_events(records, dataset, &events);

    Ok(AssemblyExtract {
        events,
        assembly_events,
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


fn extract_assembly_events(records: MatchedRecords, dataset: &Dataset, events: &Vec<Event>) -> Vec<AssemblyEvent>
{
    info!(total=records.len(), "Extracting assembly events");

    let assemblies = (records, events).into_par_iter().map(|(record, event)| {
        let (sequence, row) = record;

        AssemblyEvent {
            id: Uuid::new_v4(),
            sequence_id: sequence.id.clone(),
            event_id: event.id.clone(),

            name: row.assembly_name,
            version_status: row.version_status,
            quality: row.assembly_quality,
        }
    }).collect::<Vec<AssemblyEvent>>();

    info!(assembly_events=assemblies.len(), "Extracting assembly events finished");
    assemblies
}
