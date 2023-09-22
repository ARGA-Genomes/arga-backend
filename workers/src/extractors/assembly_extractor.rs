use std::path::PathBuf;

use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Dataset, AssemblyEvent};
use crate::error::Error;
use crate::matchers::sequence_matcher::{SequenceMatch, SequenceRecord, SequenceMap, sequence_map, match_records_mapped};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SequenceMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    sequence_record_id: String,
    assembled_by: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,

    // assembly block
    assembly_name: Option<String>,
    version_status: Option<String>,
    assembly_quality: Option<String>,
    assembly_type: Option<String>,
    genome_size: Option<i64>,
}

impl From<Record> for SequenceRecord {
    fn from(value: Record) -> Self {
        Self { record_id: value.sequence_record_id }
    }
}


pub struct AssemblyExtract {
    pub assembly_events: Vec<AssemblyEvent>,
}


pub struct AssemblyExtractIterator {
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
            Some(extract_chunk(records, &self.sequences))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<AssemblyExtractIterator, Error> {
    let sequences = sequence_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(AssemblyExtractIterator {
        sequences,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, sequences: &SequenceMap) -> Result<AssemblyExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, sequences);
    let assembly_events = extract_assembly_events(records);

    Ok(AssemblyExtract {
        assembly_events,
    })
}


fn extract_assembly_events(records: MatchedRecords) -> Vec<AssemblyEvent> {
    info!(total=records.len(), "Extracting assembly events");

    let assemblies = records.into_par_iter().map(|record| {
        let (sequence, row) = record;

        AssemblyEvent {
            id: Uuid::new_v4(),
            sequence_id: sequence.id.clone(),
            event_date: row.event_date,
            event_time: row.event_time,
            assembled_by: row.assembled_by,
            name: row.assembly_name,
            version_status: row.version_status,
            quality: row.assembly_quality,
            assembly_type: row.assembly_type,
            genome_size: row.genome_size,
        }
    }).collect::<Vec<AssemblyEvent>>();

    info!(assembly_events=assemblies.len(), "Extracting assembly events finished");
    assemblies
}
