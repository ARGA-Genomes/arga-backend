use chrono::{NaiveDate, NaiveTime};
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema;


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::libraries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Library {
    pub entity_id: String,
    pub extract_id: String,
    pub species_name_id: i64,
    pub publication_id: Option<String>,
    pub library_id: String,

    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub prepared_by: Option<String>,
    pub concentration: Option<f64>,
    pub concentration_unit: Option<String>,
    pub pcr_cycles: Option<i32>,
    pub layout: Option<String>,
    pub selection: Option<String>,
    pub bait_set_name: Option<String>,
    pub bait_set_reference: Option<String>,
    pub construction_protocol: Option<String>,
    pub source: Option<String>,
    pub insert_size: Option<String>,
    pub design_description: Option<String>,
    pub strategy: Option<String>,
    pub index_tag: Option<String>,
    pub index_dual_tag: Option<String>,
    pub index_oligo: Option<String>,
    pub index_dual_oligo: Option<String>,
    pub location: Option<String>,
    pub remarks: Option<String>,
    pub dna_treatment: Option<String>,
    pub number_of_libraries_pooled: Option<i32>,
    pub pcr_replicates: Option<i32>,
}
