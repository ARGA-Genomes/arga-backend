use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema;


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub entity_id: String,
    pub full_name: String,
    pub orcid: Option<String>,
}
