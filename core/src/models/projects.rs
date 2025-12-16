use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema;


#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::ProjectMemberRoleType"]
pub enum ProjectMemberRoleType {
    #[default]
    Lead,
}


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub entity_id: String,
    pub project_id: Option<String>,

    pub target_species_name_id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub initiative: Option<String>,
    pub registration_date: Option<chrono::NaiveDate>,

    pub data_context: Option<Vec<Option<String>>>,
    pub data_types: Option<Vec<Option<String>>>,
    pub data_assay_types: Option<Vec<Option<String>>>,
    pub partners: Option<Vec<Option<String>>>,
}


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::project_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectMember {
    pub project_entity_id: String,
    pub agent_entity_id: String,

    pub organisation: Option<String>,
    pub project_role: ProjectMemberRoleType,
}
