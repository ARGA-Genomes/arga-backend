use std::path::PathBuf;

use chrono::NaiveDate;
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Event, Dataset, AnnotationEvent};
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
    accession: Option<String>,

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
    genome_representation: Option<String>,
    genome_release_type: Option<String>,
    sequencing_coverage: Option<String>,
    num_replicons: Option<i64>,
    sop: Option<String>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}


pub struct AnnotationExtract {
    pub events: Vec<Event>,
    pub annotation_events: Vec<AnnotationEvent>,
}


pub struct AnnotationExtractIterator {
    dataset: Dataset,
    names: NameMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for AnnotationExtractIterator {
    type Item = Result<AnnotationExtract, Error>;

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
            Some(extract_chunk(records, &self.dataset, &self.names))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<AnnotationExtractIterator, Error> {
    let names = name_map(pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(AnnotationExtractIterator {
        dataset: dataset.clone(),
        names,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, names: &NameMap) -> Result<AnnotationExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, names)?;

    let events = extract_events(&records);
    let annotation_events = extract_assembly_events(records, dataset, &events);

    Ok(AnnotationExtract {
        events,
        annotation_events,
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


fn extract_assembly_events(records: MatchedRecords, dataset: &Dataset, events: &Vec<Event>) -> Vec<AnnotationEvent>
{
    info!(total=records.len(), "Extracting annotation events");

    let annotations = (records, events).into_par_iter().map(|(record, event)| {
        let (name, row) = record;

        AnnotationEvent {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            event_id: event.id.clone(),
            name_id: name.id,

            accession: row.accession,
            representation: row.genome_representation,
            release_type: row.genome_release_type,
            coverage: row.sequencing_coverage,
            replicons: row.num_replicons,
            standard_operating_procedures: row.sop,
        }
    }).collect::<Vec<AnnotationEvent>>();

    info!(annotation_events=annotations.len(), "Extracting annotation events finished");
    annotations
}
