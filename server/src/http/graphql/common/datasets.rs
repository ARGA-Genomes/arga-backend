use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models;


#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct DatasetDetails {
    pub id: Uuid,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub citation: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<models::Dataset> for DatasetDetails {
    fn from(value: models::Dataset) -> Self {
        Self {
            id: value.id,
            name: value.name,
            short_name: value.short_name,
            description: value.description,
            url: value.url,
            citation: value.citation,
            license: value.license,
            rights_holder: value.rights_holder,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct DatasetVersion {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub imported_at: DateTime<Utc>,
}

impl From<models::DatasetVersion> for DatasetVersion {
    fn from(value: models::DatasetVersion) -> Self {
        Self {
            id: value.id,
            dataset_id: value.dataset_id,
            version: value.version,
            created_at: value.created_at,
            imported_at: value.imported_at,
        }
    }
}
