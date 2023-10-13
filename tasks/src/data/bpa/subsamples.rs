use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,

    sample_submission_date: Option<String>,
    sample_id: Option<String>,
    specimen_id: Option<String>,
    tissue_number: Option<String>,
    voucher_or_tissue_number: Option<String>,
    voucher_number: Option<String>,
    voucher_herbarium_catalog_number: Option<String>,

    tissue_collection: Option<String>,
    tissue_preservation: Option<String>,
    tissue_preservation_temperature: Option<String>,
    preservation_date_begin: Option<String>,
    tissue: Option<String>,
    sample_collection_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubsampleEvent {
    id: String,
    record_id: String,
    specimen_id: String,
    material_sample_id: Option<String>,
    event_date: Option<String>,

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

        // the specimen id links to the record_id from the collections/accession events. this allows us
        // to link the higher data type subsamples to specimens
        let specimen_id = record
            .voucher_number
            .or(record.voucher_herbarium_catalog_number)
            .or(record.specimen_id)
            .or(record.sample_id.clone())
            .unwrap_or(record.id.clone());

        let material_sample_id = record
            .tissue_number
            .or(record.voucher_or_tissue_number);

        // let record_id = material_sample_id.clone().unwrap_or(specimen_id.clone());
        let record_id = match &material_sample_id {
            Some(material_id) => format!("{} {material_id}", specimen_id),
            None => specimen_id.clone(),
        };

        let event = SubsampleEvent {
            id: record.id.clone(),
            record_id,
            specimen_id,
            material_sample_id,
            event_date: record.sample_submission_date,
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
