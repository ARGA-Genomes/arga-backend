use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models;

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::AccessRightsStatus")]
pub enum AccessRightsStatus {
    Open,
    Restricted,
    Conditional,
    Variable,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::DataReuseStatus")]
pub enum DataReuseStatus {
    Limited,
    Unlimited,
    None,
    Variable,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::SourceContentType")]
pub enum SourceContentType {
    TaxonomicBackbone,
    EcologicalTraits,
    GenomicData,
    Specimens,
    NongenomicData,
    MorphologicalTraits,
    BiochemicalTraits,
    MixedDatatypes,
    FunctionalTraits,
    Ethnobiology,
}

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
    pub reuse_pill: Option<DataReuseStatus>,
    pub access_pill: Option<AccessRightsStatus>,
    pub publication_year: Option<i16>,
    pub content_type: Option<SourceContentType>,
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
            reuse_pill: value.reuse_pill.map(|r| r.into()),
            access_pill: value.access_pill.map(|a| a.into()),
            publication_year: value.publication_year.map(|p| p.into()),
            content_type: value.content_type.map(|c| c.into()),
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
