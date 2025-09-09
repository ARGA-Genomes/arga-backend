use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema;


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::dna_extracts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DnaExtract {
    pub subsample_id: String,
    pub entity_id: String,
    pub species_name_id: i64,
    pub publication_id: Option<String>,
    pub extract_id: String,
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
