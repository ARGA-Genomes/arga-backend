use arga_core::models;
use async_graphql::*;


#[derive(SimpleObject)]
pub struct Agent {
    pub entity_id: String,
    pub full_name: String,
    pub orcid: Option<String>,
}

impl From<models::Agent> for Agent {
    fn from(value: models::Agent) -> Self {
        Self {
            entity_id: value.entity_id,
            full_name: value.full_name,
            orcid: value.orcid,
        }
    }
}
