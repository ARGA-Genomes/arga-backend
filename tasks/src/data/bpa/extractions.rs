use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    bpa_sample_id: Option<String>,

    // subsample_id construction
    tissue_number: Option<String>,
    voucher_or_tissue_number: Option<String>,
    voucher_number: Option<String>,
    voucher_herbarium_catalog_number: Option<String>,
    sample_id: Option<String>,
    specimen_id: Option<String>,

    dna_extraction_date: Option<String>,
    genomic_material_preparation_date: Option<String>,
    material_extraction_date: Option<String>,

    dna_concentration: Option<String>,
    dna_conc_ng_ul: Option<String>,
    material_conc_ng_ul: Option<String>,

    dna_concentration_method: Option<String>,
    absorbance_260_230_ratio: Option<String>,
    absorbance_260_280_ratio: Option<String>,

    dna_extract: Option<String>,
    material_extraction_method: Option<String>,
    dna_extraction_method: Option<String>,
    extract_protocol: Option<String>,
    dna_extraction_protocol: Option<String>,

    dna_extracted_by: Option<String>,
    material_extracted_by: Option<String>,

    preservation_type: Option<String>,
    material_extraction_type: Option<String>,
    sample_quality: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnaExtractionEvent {
    id: String,
    record_id: String,
    subsample_id: String,
    event_date: Option<String>,
    concentration: Option<String>,
    concentration_method: Option<String>,
    absorbance_260_230: Option<String>,
    absorbance_260_280: Option<String>,
    dna_extraction_method: Option<String>,
    extracted_by: Option<String>,
    preservation_type: Option<String>,
    preparation_type: Option<String>,
    quality: Option<String>,
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("extractions.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let record_id = record
            .bpa_sample_id
            .or(record.sample_id.clone())
            .unwrap_or(record.id.clone());

        let specimen_id = record
            .voucher_number
            .or(record.voucher_herbarium_catalog_number)
            .or(record.specimen_id)
            .or(record.sample_id.clone())
            .unwrap_or(record.id.clone());

        let subsample_material_id = record.tissue_number.or(record.voucher_or_tissue_number);
        let subsample_id = match subsample_material_id {
            Some(material_id) => format!("{} {material_id}", specimen_id),
            None => specimen_id,
        };

        let event_date = record
            .dna_extraction_date
            .or(record.genomic_material_preparation_date)
            .or(record.material_extraction_date);

        let concentration = record
            .dna_concentration
            .or(record.dna_conc_ng_ul)
            .or(record.material_conc_ng_ul);

        let dna_extraction_method = record
            .dna_extract
            .or(record.material_extraction_method)
            .or(record.dna_extraction_method)
            .or(record.extract_protocol)
            .or(record.dna_extraction_protocol);

        let extracted_by = record.dna_extracted_by.or(record.material_extracted_by);

        let event = DnaExtractionEvent {
            id: record.id,
            record_id,
            subsample_id,
            event_date,
            concentration,
            concentration_method: record.dna_concentration_method,
            absorbance_260_230: record.absorbance_260_230_ratio,
            absorbance_260_280: record.absorbance_260_280_ratio,
            dna_extraction_method,
            extracted_by,
            preservation_type: record.preservation_type,
            preparation_type: record.material_extraction_type,
            quality: record.sample_quality,
        };

        writer.serialize(event)?;
    }

    Ok(())
}
