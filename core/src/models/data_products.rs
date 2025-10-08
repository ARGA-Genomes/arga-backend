use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema;


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::data_products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DataProduct {
    pub entity_id: String,
    pub publication_id: Option<String>,
    pub organism_id: Option<String>,
    pub extract_id: Option<String>,
    pub sequence_run_id: Option<String>,
    pub custodian: Option<String>,

    pub sequence_sample_id: Option<String>,
    pub sequence_analysis_id: Option<String>,
    pub notes: Option<String>,
    pub context: Option<String>,
    pub type_: Option<String>,
    pub file_type: Option<String>,
    pub url: Option<String>,
    pub licence: Option<String>,
    pub access: Option<String>,
}
