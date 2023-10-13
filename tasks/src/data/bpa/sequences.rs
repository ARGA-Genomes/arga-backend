use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,

    bioplatforms_library_id: Option<String>,
    facility_sample_id: Option<String>,

    bpa_library_id: Option<String>,
    library_id: Option<String>,

    // dna extraction id generation
    bpa_sample_id: Option<String>,
    sample_id: Option<String>,

    ddrad_dataset_ids: Option<String>,
    exon_capture_dataset_ids: Option<String>,
    facility_project_code: Option<String>,
    run_date: Option<String>,
    library_location: Option<String>,
    target: Option<String>,

    sequencer: Option<String>,
    sequencing_technology: Option<String>,
    seq_technology: Option<String>,

    analytical_platform: Option<String>,
    sequencing_kit_chemistry_version: Option<String>,

    bait_set_name: Option<String>,
    bait_set_reference: Option<String>,

    facility: Option<String>,
    sequencing_facility: Option<String>,

    library_construction_protocol: Option<String>,
    library_strategy: Option<String>,
    analysis_description: Option<String>,

    analysissoftwareversion: Option<String>,
    software_version: Option<String>,
    conversion_software: Option<String>,
    analysis_software: Option<String>,

    library_ng_ul: Option<String>,
    library_prepared_by: Option<String>,
    sequence_length: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SequencingEvent {
    id: String,
    record_id: String,
    dna_extract_id: String,
    material_sample_ids: String,
    dataset_ids: String,
    institution_code: Option<String>,
    event_date: Option<String>,
    location_id: Option<String>,
    target_gene: Option<String>,
    seq_meth: Option<String>,
    sequencing_method: Option<String>,
    bait_set_name: Option<String>,
    bait_set_reference: Option<String>,
    sequencing_facility: Option<String>,
    library_protocol: Option<String>,
    library_strategy: Option<String>,
    analysis_description: Option<String>,
    sequencing_analysis_software: Option<String>,
    concentration: Option<String>,
    sequenced_by: Option<String>,
    estimated_size: Option<String>,
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("sequences.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let record_id = record
            .bpa_library_id.clone()
            .or(record.library_id)
            .unwrap_or(record.id.clone());

        let dna_extract_id = record
            .bpa_sample_id
            .or(record.sample_id.clone())
            .unwrap_or(record.id.clone());

        let sample_ids: Vec<String> = vec![
            record.bioplatforms_library_id,
            record.bpa_library_id,
            record.facility_sample_id,
        ].into_iter().filter_map(|r| r).collect();

        let dataset_ids: Vec<String> = vec![
            record.ddrad_dataset_ids,
            record.exon_capture_dataset_ids,
        ].into_iter().filter_map(|r| r).collect();

        let seq_meth = record.sequencer.or(record.sequencing_technology).or(record.seq_technology);
        let sequencing_method = record.analytical_platform.or(record.sequencing_kit_chemistry_version);
        let sequencing_facility = record.sequencing_facility.or(record.facility);

        let sequencing_analysis_software = record
            .analysissoftwareversion
            .or(record.software_version)
            .or(record.conversion_software)
            .or(record.analysis_software);

        let event = SequencingEvent {
            id: record.id,
            record_id,
            dna_extract_id,
            material_sample_ids: sample_ids.join(" | ").to_string(),
            dataset_ids: dataset_ids.join(" | ").to_string(),
            institution_code: record.facility_project_code,
            event_date: record.run_date,
            location_id: record.library_location,
            target_gene: record.target,
            seq_meth,
            sequencing_method,
            bait_set_name: record.bait_set_name,
            bait_set_reference: record.bait_set_reference,
            sequencing_facility,
            library_protocol: record.library_construction_protocol,
            library_strategy: record.library_strategy,
            analysis_description: record.analysis_description,
            sequencing_analysis_software,
            concentration: record.library_ng_ul,
            sequenced_by: record.library_prepared_by,
            estimated_size: record.sequence_length,
        };

        writer.serialize(event)?;
    }

    Ok(())
}
