use std::path::PathBuf;

use arga_core::models::{AccessionEvent, Dataset};
use csv::DeserializeRecordsIntoIter;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;
use crate::matchers::specimen_matcher::{
    match_records_mapped,
    specimen_map,
    SpecimenMap,
    SpecimenMatch,
    SpecimenRecord,
};


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(SpecimenMatch, Record)>;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,
    accession: String,
    entity_id: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    type_status: Option<String>,
    material_sample_id: Option<String>,
    event_date: Option<String>,
    event_time: Option<String>,
    accessioned_by: Option<String>,
}

impl From<Record> for SpecimenRecord {
    fn from(value: Record) -> Self {
        Self {
            record_id: value.record_id,
        }
    }
}


pub struct AccessionExtract {
    pub accession_events: Vec<AccessionEvent>,
}


pub struct AccessionExtractIterator {
    dataset: Dataset,
    specimens: SpecimenMap,
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
            Some(extract_chunk(records, &self.dataset, &self.specimens))
        }
    }
}


/// Extract accession events and other related data from a CSV file
pub fn extract(
    path: PathBuf,
    dataset: &Dataset,
    context: &Vec<Dataset>,
    pool: &mut PgPool,
) -> Result<AccessionExtractIterator, Error> {
    let isolated_datasets = context.iter().map(|d| d.id.clone()).collect();

    let specimens = specimen_map(&isolated_datasets, pool)?;
    let reader = csv::Reader::from_path(&path)?.into_deserialize();

    Ok(AccessionExtractIterator {
        dataset: dataset.clone(),
        specimens,
        reader,
    })
}


fn extract_chunk(chunk: Vec<Record>, dataset: &Dataset, specimens: &SpecimenMap) -> Result<AccessionExtract, Error> {
    // match the records to specimens in the database. this will filter out any accessions
    // that could not be matched
    let records = match_records_mapped(chunk, specimens);
    let accession_events = extract_accession_events(dataset, records);

    Ok(AccessionExtract { accession_events })
}


fn extract_accession_events(dataset: &Dataset, records: MatchedRecords) -> Vec<AccessionEvent> {
    info!(total = records.len(), "Extracting accession events");

    let accessions = records
        .into_par_iter()
        .map(|record| {
            let (specimen, row) = record;

            AccessionEvent {
                id: Uuid::new_v4(),
                dataset_id: dataset.id.clone(),
                specimen_id: specimen.id,
                entity_id: row.entity_id,
                event_date: row.event_date,
                event_time: row.event_time,
                accession: row.accession,
                accessioned_by: row.accessioned_by,
                material_sample_id: row.material_sample_id,
                institution_name: row.institution_name,
                institution_code: row.institution_code,
                type_status: row.type_status,
            }
        })
        .collect::<Vec<AccessionEvent>>();

    info!(accession_events = accessions.len(), "Extracting accession events finished");
    accessions
}
