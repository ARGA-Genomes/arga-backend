use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,

    sample_submission_date: Option<String>,
    tissue_number: Option<String>,
    voucher_or_tissue_number: Option<String>,

    tissue_collection: Option<String>,
    tissue_preservation: Option<String>,
    tissue_preservation_temperature: Option<String>,
    preservation_date_begin: Option<String>,
    tissue: Option<String>,
    sample_collection_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubsampleEvent {
    record_id: String,
    event_date: Option<String>,
    material_sample_id: Option<String>,

    subsampled_by: Option<String>,
    preservation_type: Option<String>,
    preservation_temperature: Option<String>,
    preservation_date_begin: Option<String>,
    preparation_type: Option<String>,
    material_sample_type: Option<String>,
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("subsamples.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let material_sample_id = record.tissue_number.or(record.voucher_or_tissue_number);

        let event = SubsampleEvent {
            record_id: record.id,
            event_date: record.sample_submission_date,
            material_sample_id,
            subsampled_by: record.tissue_collection,
            preservation_type: record.tissue_preservation,
            preservation_temperature: record.tissue_preservation_temperature,
            preservation_date_begin: record.preservation_date_begin,
            preparation_type: record.tissue,
            material_sample_type: record.sample_collection_type,
        };

        writer.serialize(event)?;
    }

    Ok(())
}
