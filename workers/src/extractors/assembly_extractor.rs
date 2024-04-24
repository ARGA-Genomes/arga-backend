use std::path::PathBuf;

use arga_core::models::{AssemblyEvent, Dataset};
use csv::DeserializeRecordsIntoIter;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;
use crate::matchers::sequence_matcher::{
    match_records_mapped,
    sequence_map,
    SequenceMap,
    SequenceMatch,
    SequenceRecord,
};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SequenceMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    sequence_id: String,
    // record_id: String,
    // sequence_record_id: Option<String>,
    entity_id: Option<String>,
    assembled_by: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,

    // assembly block
    assembly_name: Option<String>,
    version_status: Option<String>,
    assembly_quality: Option<String>,
    assembly_type: Option<String>,
    genome_size: Option<String>,
}

impl From<Record> for SequenceRecord {
    fn from(value: Record) -> Self {
        Self {
            record_id: value.sequence_id,
        }
        // Self { record_id: value.sequence_record_id.unwrap_or(value.record_id) }
    }
}


pub struct AssemblyExtract {
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
            Some(extract_chunk(records, &self.dataset, &self.sequences))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<AssemblyExtractIterator, Error> {
    let sequences = sequence_map(&vec![dataset.id], pool)?;
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
    let assembly_events = extract_assembly_events(dataset, records);

    Ok(AssemblyExtract { assembly_events })
}


fn extract_assembly_events(dataset: &Dataset, records: MatchedRecords) -> Vec<AssemblyEvent> {
    info!(total = records.len(), "Extracting assembly events");

    let assemblies = records
        .into_par_iter()
        .map(|record| {
            let (sequence, row) = record;

            AssemblyEvent {
                id: Uuid::new_v4(),
                dataset_id: dataset.id.clone(),
                sequence_id: sequence.id.clone(),
                entity_id: row.entity_id,
                event_date: row.event_date,
                event_time: row.event_time,
                assembled_by: row.assembled_by,
                name: row.assembly_name,
                version_status: row.version_status,
                quality: row.assembly_quality,
                assembly_type: row.assembly_type,
                genome_size: parse_i64(row.genome_size),
            }
        })
        .collect::<Vec<AssemblyEvent>>();

    info!(assembly_events = assemblies.len(), "Extracting assembly events finished");
    assemblies
}


fn parse_i64(value: Option<String>) -> Option<i64> {
    match value {
        Some(v) => str::parse::<i64>(&v).ok(),
        None => None,
    }
}
