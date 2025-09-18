use async_graphql::*;

use crate::database::models;


#[derive(Clone, Debug, SimpleObject)]
pub struct DnaExtractDetails {
    pub entity_id: String,
    pub extract_id: String,
    pub subsample_id: String,

    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub extracted_by: Option<String>,
    pub material_extracted_by: Option<String>,
    pub nucleic_acid_type: Option<String>,
    pub preparation_type: Option<String>,
    pub preservation_type: Option<String>,
    pub preservation_method: Option<String>,
    pub extraction_method: Option<String>,
    pub concentration_method: Option<String>,
    pub conformation: Option<String>,
    pub concentration: Option<f64>,
    pub concentration_unit: Option<String>,
    pub quantification: Option<String>,
    pub absorbance_260_230_ratio: Option<f64>,
    pub absorbance_260_280_ratio: Option<f64>,
    pub cell_lysis_method: Option<String>,
    pub action_extracted: Option<String>,
    pub number_of_extracts_pooled: Option<String>,
}

impl From<models::DnaExtract> for DnaExtractDetails {
    fn from(value: models::DnaExtract) -> Self {
        Self {
            entity_id: value.entity_id,
            extract_id: value.extract_id,
            subsample_id: value.subsample_id,
            event_date: value.event_date,
            event_time: value.event_time,
            extracted_by: value.extracted_by,
            material_extracted_by: value.material_extracted_by,
            nucleic_acid_type: value.nucleic_acid_type,
            preparation_type: value.preparation_type,
            preservation_type: value.preservation_type,
            preservation_method: value.preservation_method,
            extraction_method: value.extraction_method,
            concentration_method: value.concentration_method,
            conformation: value.conformation,
            concentration: value.concentration,
            concentration_unit: value.concentration_unit,
            quantification: value.quantification,
            absorbance_260_230_ratio: value.absorbance_260_230_ratio,
            absorbance_260_280_ratio: value.absorbance_260_280_ratio,
            cell_lysis_method: value.cell_lysis_method,
            action_extracted: value.action_extracted,
            number_of_extracts_pooled: value.number_of_extracts_pooled,
        }
    }
}
