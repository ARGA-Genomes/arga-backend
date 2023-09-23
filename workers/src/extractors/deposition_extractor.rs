use std::path::PathBuf;

use chrono::NaiveDate;
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Dataset, DepositionEvent};
use crate::error::Error;
use crate::matchers::sequence_matcher::{SequenceMatch, SequenceRecord, SequenceMap, sequence_map, match_records_mapped};

use super::utils::naive_date_from_str_opt;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SequenceMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,
    sequence_record_id: Option<String>,
    accession: Option<String>,
    material_sample_id: Option<String>,
    submitted_by: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,

    // deposition block
    collection_name: Option<String>,
    collection_code: Option<String>,
    institution_name: Option<String>,
    data_type: Option<String>,
    excluded_from_refseq: Option<String>,
    asm_not_live_date: Option<String>,
    source_uri: Option<String>,

    title: Option<String>,
    url: Option<String>,
    funding_attribution: Option<String>,
    rights_holder: Option<String>,
    access_rights: Option<String>,
    reference: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "naive_date_from_str_opt")]
    last_updated: Option<NaiveDate>,
}

impl From<Record> for SequenceRecord {
    fn from(value: Record) -> Self {
        Self { record_id: value.sequence_record_id.unwrap_or(value.record_id) }
    }
}


pub struct DepositionExtract {
    pub deposition_events: Vec<DepositionEvent>,
}


pub struct DepositionExtractIterator {
    sequences: SequenceMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for DepositionExtractIterator {
    type Item = Result<DepositionExtract, Error>;

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
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<DepositionExtractIterator, Error> {
    let sequences = sequence_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(DepositionExtractIterator {
        sequences,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, sequences: &SequenceMap) -> Result<DepositionExtract, Error> {
    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records_mapped(chunk, sequences);
    let deposition_events = extract_deposition_events(records);

    Ok(DepositionExtract {
        deposition_events,
    })
}


fn extract_deposition_events(records: MatchedRecords) -> Vec<DepositionEvent>
{
    info!(total=records.len(), "Extracting deposition events");

    let depositions = records.into_par_iter().map(|record| {
        let (sequence, row) = record;

        DepositionEvent {
            id: Uuid::new_v4(),
            sequence_id: sequence.id.clone(),

            event_date: row.event_date,
            event_time: row.event_time,
            accession: row.accession,
            submitted_by: row.submitted_by,

            material_sample_id: row.material_sample_id,
            collection_name: row.collection_name,
            collection_code: row.collection_code,
            institution_name: row.institution_name,

            data_type: row.data_type,
            excluded_from_refseq: row.excluded_from_refseq,
            asm_not_live_date: row.asm_not_live_date,
            source_uri: row.source_uri,

            title: row.title,
            url: row.url,
            funding_attribution: row.funding_attribution,
            rights_holder: row.rights_holder,
            access_rights: row.access_rights,
            reference: row.reference,
            last_updated: row.last_updated,
        }
    }).collect::<Vec<DepositionEvent>>();

    info!(deposition_events=depositions.len(), "Extracting deposition events finished");
    depositions
}
