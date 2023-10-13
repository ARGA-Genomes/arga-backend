use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,

    // museum catalog numbers
    voucher_number: Option<String>,
    voucher_herbarium_record_number: Option<String>,
    voucher_herbarium_catalog_number: Option<String>,
    voucher_or_tissue_number: Option<String>,
    voucher_id: Option<String>,
    specimen_id: Option<String>,

    sample_id: Option<String>,

    type_status: Option<String>,
    institution_name: Option<String>,
    voucher_herbarium_event_date: Option<String>,
    voucher_herbarium_recorded_by: Option<String>,
    sample_quality: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessionEvent {
    id: String,
    specimen_id: String,
    accession: String,
    material_sample_ids: String,
    type_status: Option<String>,
    institution_code: Option<String>,
    event_date: Option<String>,
    accessioned_by: Option<String>,
    quality: Option<String>,
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("accessions.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let specimen_id = record
            .voucher_number.clone()
            .or(record.voucher_herbarium_catalog_number.clone())
            .or(record.specimen_id)
            .or(record.sample_id.clone())
            .unwrap_or(record.id.clone());

        let accession = specimen_id.clone();

        // let accession = record.voucher_herbarium_catalog_number.as_ref()
        //     .or(record.voucher_herbarium_record_number.as_ref())
        //     .or(record.voucher_or_tissue_number.as_ref())
        //     .or(record.voucher_id.as_ref())
        //     .or(record.voucher_number.as_ref())
        //     .unwrap_or(&specimen_id)
        //     .clone();

        let vouchers: Vec<String> = vec![
            record.voucher_herbarium_catalog_number,
            record.voucher_herbarium_record_number,
            record.voucher_or_tissue_number,
            record.voucher_id,
            record.voucher_number,
        ].into_iter().filter_map(|r| r).collect();


        let event = AccessionEvent {
            id: record.id,
            specimen_id,
            accession,
            material_sample_ids: vouchers.join(" | ").to_string(),
            type_status: record.type_status,
            institution_code: record.institution_name,
            event_date: record.voucher_herbarium_event_date,
            accessioned_by: record.voucher_herbarium_recorded_by,
            quality: record.sample_quality,
        };

        writer.serialize(event)?;
    }

    Ok(())
}
