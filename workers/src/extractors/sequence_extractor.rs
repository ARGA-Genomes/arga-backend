use std::path::PathBuf;

use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use itertools::izip;
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{SequencingEvent, Dataset, Sequence, SequencingRunEvent};
use crate::error::Error;
use crate::extractors::utils::parse_naive_date_time;
use crate::matchers::dna_extract_matcher::{DnaExtractMatch, DnaExtractRecord, DnaExtractMap, dna_extract_map, match_records_mapped};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(DnaExtractMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,
    dna_extract_id: String,
    sequence_record_id: Option<String>,
    material_sample_id: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,

    sequenced_by: Option<String>,
    target_gene: Option<String>,
    dna_sequence: Option<String>,

    concentration: Option<String>,
    amplicon_size: Option<i64>,
    estimated_size: Option<String>,
    bait_set_name: Option<String>,
    bait_set_reference: Option<String>,

    // sequence run
    trace_ids: Option<String>,
    trace_names: Option<String>,
    trace_links: Option<String>,
    run_dates: Option<String>,
    sequencing_centers: Option<String>,
    directions: Option<String>,
    sequence_primers: Option<String>,
    marker_codes: Option<String>,
}

impl From<Record> for DnaExtractRecord {
    fn from(value: Record) -> Self {
        return Self { record_id: value.dna_extract_id };
        // a dataset can be made up of multiple different datasets
        // which might have different accessioned IDs for different stages.
        // for example, with NCBI the event chain starts with BioSamples
        // that have an ID of SAMNxxx, and later is referenced in Genbank and RefSeq
        // via the matierial sample id, and instead having its own accession
        // id of GCAxxx and GCFxxx respectively
        // let record_id = match value.material_sample_id {
        //     Some(sample_id) => sample_id,
        //     None => value.record_id,
        // };
        // Self { record_id }
    }
}


pub struct SequencingExtract {
    pub sequences: Vec<Sequence>,
    pub sequencing_events: Vec<SequencingEvent>,
    pub sequencing_run_events: Vec<SequencingRunEvent>,
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
pub fn extract(path: PathBuf, dataset: &Dataset, context: &Vec<Dataset>, pool: &mut PgPool) -> Result<SequencingExtractIterator, Error> {
    // we want to limit the data we match on but also have datasets that reference
    // each other like the NCBI datasets, so we allow an isolated context of datasets
    // that can be used to match on to enable more complex import scenarios
    let isolated_datasets = context.iter().map(|d| d.id.clone()).collect();

    let dna_extracts = dna_extract_map(&isolated_datasets, pool)?;
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

    let sequences = extract_sequences(dataset, &records);
    let sequencing_events = extract_sequencing_events(dataset, &records, &sequences);
    let sequencing_run_events = extract_sequencing_run_events(records, &sequencing_events);

    Ok(SequencingExtract {
        sequences,
        sequencing_events,
        sequencing_run_events,
    })
}


fn extract_sequences(dataset: &Dataset, records: &MatchedRecords) -> Vec<Sequence> {
    info!(total=records.len(), "Extracting sequences");

    let sequences = records.par_iter().map(|(dna_extract, row)| {
        // sometimes a record can have a more descriptive identifier and we
        // want to leverage that when possible, so we fallback to a mandatory
        // identifier when a more granular one doesn't exist
        let record_id = match &row.sequence_record_id {
            Some(id) => id.clone(),
            None => row.record_id.clone(),
        };

        Sequence {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            name_id: dna_extract.name_id.clone(),
            dna_extract_id: dna_extract.id.clone(),
            record_id,
        }
    }).collect::<Vec<Sequence>>();

    info!(sequences=sequences.len(), "Extracting sequences finished");
    sequences
}


fn extract_sequencing_events(dataset: &Dataset, records: &MatchedRecords, sequences: &Vec<Sequence>) -> Vec<SequencingEvent> {
    info!(total=records.len(), "Extracting sequencing events");

    let sequences = (records, sequences).into_par_iter().map(|(record, sequence)| {
        let (_subsample, row) = record;

        SequencingEvent {
            id: Uuid::new_v4(),
            dataset_id: dataset.id.clone(),
            sequence_id: sequence.id.clone(),

            event_date: row.event_date.clone(),
            event_time: row.event_time.clone(),
            sequenced_by: row.sequenced_by.clone(),
            material_sample_id: row.material_sample_id.clone(),

            concentration: parse_f64(row.concentration.clone()),
            amplicon_size: row.amplicon_size,
            estimated_size: row.estimated_size.clone(),
            bait_set_name: row.bait_set_name.clone(),
            bait_set_reference: row.bait_set_reference.clone(),

            target_gene: row.target_gene.clone(),
            dna_sequence: row.dna_sequence.clone(),
        }
    }).collect::<Vec<SequencingEvent>>();

    info!(sequencing_events=sequences.len(), "Extracting sequencing events finished");
    sequences
}


fn extract_sequencing_run_events(
    records: MatchedRecords,
    events: &Vec<SequencingEvent>
) -> Vec<SequencingRunEvent>
{
    info!(total=records.len(), "Extracting sequencing run events");

    let runs = (records, events).into_par_iter().map(|(record, event)| {
        let (_subsample, row) = record;

        let trace_ids = str_to_vec(&row.trace_ids);
        let trace_names = str_to_vec(&row.trace_names);
        let trace_links = str_to_vec(&row.trace_links);
        let run_dates = str_to_vec(&row.run_dates);
        let sequencing_centers = str_to_vec(&row.sequencing_centers);
        let directions = str_to_vec(&row.directions);
        let sequence_primers = str_to_vec(&row.sequence_primers);
        let marker_codes = str_to_vec(&row.marker_codes);

        let mut run_events = Vec::new();

        for (trace_id, name, link, run_date, sequencing_center, direction, primer, target_gene) in izip!(
            trace_ids,
            trace_names,
            trace_links,
            run_dates,
            sequencing_centers,
            directions,
            sequence_primers,
            marker_codes,
        ) {
            run_events.push(SequencingRunEvent {
                id: Uuid::new_v4(),
                sequencing_event_id: event.id,
                trace_id: Some(trace_id),
                trace_name: Some(name),
                trace_link: Some(link),
                sequencing_date: parse_naive_date_time(&run_date).ok(),
                sequencing_center: Some(sequencing_center),
                sequencing_center_code: None,
                sequencing_method: None,
                target_gene: Some(target_gene),
                direction: Some(direction),
                pcr_primer_name_forward: None,
                pcr_primer_name_reverse: None,
                sequence_primer_forward_name: Some(primer),
                sequence_primer_reverse_name: None,
                library_protocol: None,
                analysis_description: None,
                analysis_software: None,
            });
        }

        run_events
    }).flatten().collect::<Vec<SequencingRunEvent>>();

    info!(sequencing_run_events=runs.len(), "Extracting sequencing run events finished");
    runs
}


fn str_to_vec(value: &Option<String>) -> Vec<String> {
    match value {
        Some(val) => val.split("|").map(|v| v.to_string()).collect(),
        None => Vec::new(),
    }
}


fn parse_f64(value: Option<String>) -> Option<f64> {
    match value {
        Some(v) => str::parse::<f64>(&v).ok(),
        None => None,
    }
}
