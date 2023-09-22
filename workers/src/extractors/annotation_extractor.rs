use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Dataset, AnnotationEvent};
use crate::error::Error;
use crate::matchers::sequence_matcher::{SequenceMatch, SequenceRecord, SequenceMap, sequence_map, match_records_mapped};

use super::utils::naive_date_from_str_opt;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SequenceMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    sequence_record_id: String,
    annotated_by: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "naive_date_from_str_opt")]
    event_date: Option<NaiveDate>,
    event_time: Option<NaiveTime>,

    // assembly block
    genome_representation: Option<String>,
    genome_release_type: Option<String>,
    sequencing_coverage: Option<String>,
    num_replicons: Option<i64>,
    sop: Option<String>,
}

impl From<Record> for SequenceRecord {
    fn from(value: Record) -> Self {
        Self { record_id: value.sequence_record_id }
    }
}


pub struct AnnotationExtract {
    pub annotation_events: Vec<AnnotationEvent>,
}


pub struct AnnotationExtractIterator {
    sequences: SequenceMap,
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
            Some(extract_chunk(records, &self.sequences))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<AnnotationExtractIterator, Error> {
    let sequences = sequence_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(AnnotationExtractIterator {
        sequences,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, sequences: &SequenceMap) -> Result<AnnotationExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, sequences);
    let annotation_events = extract_annotation_events(records);

    Ok(AnnotationExtract {
        annotation_events,
    })
}


fn extract_annotation_events(records: MatchedRecords) -> Vec<AnnotationEvent>
{
    info!(total=records.len(), "Extracting annotation events");

    let annotations = records.into_par_iter().map(|record| {
        let (sequence, row) = record;

        AnnotationEvent {
            id: Uuid::new_v4(),
            sequence_id: sequence.id.clone(),
            event_date: row.event_date,
            event_time: row.event_time,
            annotated_by: row.annotated_by,
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
