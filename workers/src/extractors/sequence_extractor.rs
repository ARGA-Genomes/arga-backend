use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};
use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Event, SequencingEvent, Dataset, Sequence};
use crate::error::Error;
use crate::matchers::dna_extract_matcher::{DnaExtractMatch, DnaExtractRecord, DnaExtractMap, dna_extract_map, match_records_mapped};

use super::utils::naive_date_from_str_opt;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(DnaExtractMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    accession: String,

    // event block
    field_number: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "naive_date_from_str_opt")]
    event_date: Option<NaiveDate>,
    event_time: Option<NaiveTime>,
    habitat: Option<String>,
    sampling_protocol: Option<String>,
    sampling_size_value: Option<String>,
    sampling_size_unit: Option<String>,
    sampling_effort: Option<String>,
    field_notes: Option<String>,
    event_remarks: Option<String>,

    // dna block
    genbank_accession: Option<String>,
    material_sample_id: Option<String>,
    sequenced_by: Option<String>,
    target_gene: Option<String>,
    dna_sequence: Option<String>,

    // sequence run
    //
    // seq_method: Option<String>,
    // sequencing_method: Option<String>,
    // sequencing_center: Option<String>,
    // sequencing_center_code: Option<String>,
    // library_protocol: Option<String>,
    // analysis_description: Option<String>,
    // sequencing_analysis_software: Option<String>,

    concentration: Option<f64>,
    amplicon_size: Option<i64>,
    estimated_size: Option<i64>,
    bait_set_name: Option<String>,
    bait_set_reference: Option<String>,
}

impl From<Record> for DnaExtractRecord {
    fn from(value: Record) -> Self {
        Self { accession: value.accession }
    }
}


pub struct SequencingExtract {
    pub events: Vec<Event>,
    pub sequences: Vec<Sequence>,
    pub sequencing_events: Vec<SequencingEvent>,
}


pub struct SequencingExtractIterator {
    dataset: Dataset,
    dna_extracts: DnaExtractMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for SequencingExtractIterator {
    type Item = Result<SequencingExtract, Error>;

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
            Some(extract_chunk(records, &self.dataset, &self.dna_extracts))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<SequencingExtractIterator, Error> {
    let dna_extracts = dna_extract_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(SequencingExtractIterator {
        dataset: dataset.clone(),
        dna_extracts,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, extracts: &DnaExtractMap) -> Result<SequencingExtract, Error> {
    // match the records to dna extracts in the database. this will filter out any subsamples
    // that could not be matched
    let records = match_records_mapped(chunk, extracts);

    let events = extract_events(&records);
    let sequences = extract_sequences(&records);
    let sequencing_events = extract_sequencing_events(records, &sequences, &events);

    Ok(SequencingExtract {
        events,
        sequences,
        sequencing_events,
    })
}


fn extract_events(records: &MatchedRecords) -> Vec<Event> {
    info!(total=records.len(), "Extracting events");

    let events = records.par_iter().map(|(_name, row)| {
        Event {
            id: Uuid::new_v4(),
            field_number: row.field_number.clone(),
            event_date: row.event_date.clone(),
            event_time: row.event_time.clone(),
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


fn extract_sequences(records: &MatchedRecords) -> Vec<Sequence> {
    info!(total=records.len(), "Extracting sequences");

    let sequences = records.par_iter().map(|(dna_extract, row)| {
        Sequence {
            id: Uuid::new_v4(),
            dataset_id: dna_extract.dataset_id.clone(),
            name_id: dna_extract.name_id.clone(),
            dna_extract_id: dna_extract.id.clone(),

            accession: row.accession.clone(),
            genbank_accession: row.genbank_accession.clone(),
        }
    }).collect::<Vec<Sequence>>();

    info!(sequences=sequences.len(), "Extracting sequences finished");
    sequences
}


fn extract_sequencing_events(records: MatchedRecords, sequences: &Vec<Sequence>, events: &Vec<Event>) -> Vec<SequencingEvent> {
    info!(total=records.len(), "Extracting sequencing events");

    let sequences = (records, sequences, events).into_par_iter().map(|(record, sequence, event)| {
        let (_subsample, row) = record;

        SequencingEvent {
            id: Uuid::new_v4(),
            sequence_id: sequence.id.clone(),
            event_id: event.id.clone(),

            sequenced_by: row.sequenced_by,
            material_sample_id: row.material_sample_id,

            concentration: row.concentration,
            amplicon_size: row.amplicon_size,
            estimated_size: row.estimated_size,
            bait_set_name: row.bait_set_name,
            bait_set_reference: row.bait_set_reference,

            target_gene: row.target_gene,
            dna_sequence: row.dna_sequence,

        }
    }).collect::<Vec<SequencingEvent>>();

    info!(sequencing_events=sequences.len(), "Extracting sequencing events finished");
    sequences
}
