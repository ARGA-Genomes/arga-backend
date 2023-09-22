use std::path::PathBuf;

use csv::DeserializeRecordsIntoIter;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{Dataset, DnaExtractionEvent, DnaExtract};
use crate::error::Error;
use crate::matchers::subsample_matcher::{SubsampleMatch, SubsampleRecord, SubsampleMap, subsample_map, match_records_mapped};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SubsampleMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,

    event_date: Option<String>,
    event_time: Option<String>,

    // extraction block
    extracted_by: Option<String>,
    preservation_type: Option<String>,
    preparation_type: Option<String>,
    extraction_method: Option<String>,
    measurement_method: Option<String>,
    concentration_method: Option<String>,
    quality: Option<String>,
    concentration: Option<f64>,
    absorbance_260_230: Option<f64>,
    absorbance_260_280: Option<f64>,
}

impl From<Record> for SubsampleRecord {
    fn from(value: Record) -> Self {
        Self { record_id: value.record_id }
    }
}


pub struct DnaExtractionExtract {
    pub dna_extracts: Vec<DnaExtract>,
    pub dna_extraction_events: Vec<DnaExtractionEvent>,
}


pub struct DnaExtractionExtractIterator {
    subsamples: SubsampleMap,
    reader: DeserializeRecordsIntoIter<std::fs::File, Record>,
}

impl Iterator for DnaExtractionExtractIterator {
    type Item = Result<DnaExtractionExtract, Error>;

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
            Some(extract_chunk(records, &self.subsamples))
        }
    }
}


/// Extract events and other related data from a CSV file
pub fn extract(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<DnaExtractionExtractIterator, Error> {
    let subsamples = subsample_map(&dataset.id, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(DnaExtractionExtractIterator {
        subsamples,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, subsamples: &SubsampleMap) -> Result<DnaExtractionExtract, Error> {
    // match the records to names in the database. this will filter out any subsamples
    // that could not be matched
    let records = match_records_mapped(chunk, subsamples);

    let dna_extracts = extract_dna_extracts(&records);
    let dna_extraction_events = extract_dna_extraction_events(records, &dna_extracts);

    Ok(DnaExtractionExtract {
        dna_extracts,
        dna_extraction_events,
    })
}


fn extract_dna_extracts(records: &MatchedRecords) -> Vec<DnaExtract> {
    info!(total=records.len(), "Extracting dna extracts");

    let dna_extracts = records.par_iter().map(|(subsample, row)| {
        DnaExtract {
            id: Uuid::new_v4(),
            dataset_id: subsample.dataset_id.clone(),
            name_id: subsample.name_id.clone(),
            subsample_id: subsample.id.clone(),
            record_id: row.record_id.clone(),
        }
    }).collect::<Vec<DnaExtract>>();

    info!(dna_extracts=dna_extracts.len(), "Extracting dna extracts finished");
    dna_extracts
}


fn extract_dna_extraction_events(records: MatchedRecords, extracts: &Vec<DnaExtract>) -> Vec<DnaExtractionEvent>
{
    info!(total=records.len(), "Extracting dna extraction events");

    let extractions = (records, extracts).into_par_iter().map(|(record, extract)| {
        let (_subsample, row) = record;

        DnaExtractionEvent {
            id: Uuid::new_v4(),
            dna_extract_id: extract.id.clone(),

            event_date: row.event_date,
            event_time: row.event_time,
            extracted_by: row.extracted_by,

            preservation_type: row.preservation_type,
            preparation_type: row.preparation_type,
            extraction_method: row.extraction_method,
            measurement_method: row.measurement_method,
            concentration_method: row.concentration_method,
            quality: row.quality,
            concentration: row.concentration,
            absorbance_260_230: row.absorbance_260_230,
            absorbance_260_280: row.absorbance_260_280,
        }
    }).collect::<Vec<DnaExtractionEvent>>();

    info!(dna_extraction_events=extractions.len(), "Extracting dna extraction events finished");
    extractions
}
