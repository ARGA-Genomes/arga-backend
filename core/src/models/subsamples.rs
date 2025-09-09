use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema;


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::subsamples)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Subsample {
    pub specimen_id: String,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub entity_id: String,
    pub species_name_id: i64,
    pub publication_id: Option<String>,
    pub subsample_id: String,

    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub sample_type: Option<String>,
    pub name: Option<String>,
    pub custodian: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub culture_method: Option<String>,
    pub culture_media: Option<String>,
    pub weight_or_volume: Option<String>,
    pub preservation_method: Option<String>,
    pub preservation_temperature: Option<String>,
    pub preservation_duration: Option<String>,
    pub quality: Option<String>,
    pub cell_type: Option<String>,
    pub cell_line: Option<String>,
    pub clone_name: Option<String>,
    pub lab_host: Option<String>,
    pub sample_processing: Option<String>,
    pub sample_pooling: Option<String>,
}
