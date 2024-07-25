use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    bpa_sample_id: Option<String>,
    bpa_dataset_id: Option<String>,

    bpa_library_id: Option<String>,
    library_id: Option<String>,

    owner_org: Option<String>,
    access_rights: Option<String>,
    publication_reference: Option<String>,
    institution_name: Option<String>,

    dataset_id: Option<String>,
    bioplatforms_dataset_id: Option<String>,

    bioplatforms_project_code: Option<String>,
    bioplatforms_project: Option<String>,

    date_submission: Option<String>,
    date_data_published: Option<String>,

    ncbi_biosample_accession: Option<String>,
    ncbi_biosample_accession_number: Option<String>,

    url: Option<String>,
    // base_url: Option<String>,
    r#type: Option<String>,
    data_custodian: Option<String>,
    funding_agency: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepositionEvent {
    id: String,
    sequence_id: String,
    event_date: Option<String>,
    material_sample_id: Option<String>,
    rights_holder: Option<String>,
    access_rights: Option<String>,
    reference: Option<String>,
    institution_name: Option<String>,
    dataset_ids: String,
    collection_code: Option<String>,
    biosample_accession: Option<String>,
    project_name: Option<String>,
    url: Option<String>,
    source_uri: Option<String>,
    submitted_by: Option<String>,
    funding_attribution: Option<String>,
    title: Option<String>,
    data_type: Option<String>,
}

impl DepositionEvent {
    fn has_data(&self) -> bool {
        self.event_date.is_some()
            || self.material_sample_id.is_some()
            || self.rights_holder.is_some()
            || self.access_rights.is_some()
            || self.reference.is_some()
            || self.institution_name.is_some()
            || self.collection_code.is_some()
            || self.biosample_accession.is_some()
            || self.project_name.is_some()
            || self.url.is_some()
            || self.submitted_by.is_some()
            || self.funding_attribution.is_some()
            || self.title.is_some()
    }
}


pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("depositions.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let sequence_id = record.bpa_library_id.or(record.library_id).unwrap_or(record.id.clone());

        let event_date = record.date_submission.or(record.date_data_published);
        let biosample_accession = record
            .ncbi_biosample_accession
            .or(record.ncbi_biosample_accession_number);

        let dataset_ids: Vec<String> = vec![record.dataset_id, record.bioplatforms_dataset_id, record.bpa_dataset_id]
            .into_iter()
            .filter_map(|r| r)
            .collect();

        let source_uri = match &record.r#type {
            Some(data_type) => Some(format!("https://data.bioplatforms.com/{}/{}", data_type, record.id)),
            None => None,
        };

        let event = DepositionEvent {
            id: record.id,
            sequence_id,
            event_date,
            material_sample_id: record.bpa_sample_id,
            rights_holder: record.owner_org,
            access_rights: record.access_rights,
            reference: record.publication_reference,
            institution_name: record.institution_name,
            dataset_ids: dataset_ids.join(" | ").to_string(),
            collection_code: record.bioplatforms_project_code,
            biosample_accession,
            project_name: record.bioplatforms_project,
            url: record.url,
            source_uri,
            submitted_by: record.data_custodian,
            funding_attribution: record.funding_agency,
            title: record.title,
            data_type: record.r#type,
        };

        if event.has_data() {
            writer.serialize(event)?;
        }
    }

    Ok(())
}
