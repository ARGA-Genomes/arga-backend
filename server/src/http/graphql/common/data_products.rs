use async_graphql::*;

use crate::database::models;


#[derive(Clone, Debug, SimpleObject)]
pub struct DataProductDetails {
    pub entity_id: String,
    pub extract_id: Option<String>,
    pub sequence_run_id: Option<String>,
    pub sequence_sample_id: Option<String>,
    pub sequence_analysis_id: Option<String>,
    pub notes: Option<String>,
    pub context: Option<String>,
    pub r#type: Option<String>,
    pub file_type: Option<String>,
    pub url: Option<String>,
    pub licence: Option<String>,
    pub access: Option<String>,
}

impl From<models::DataProduct> for DataProductDetails {
    fn from(value: models::DataProduct) -> Self {
        DataProductDetails {
            entity_id: value.entity_id,
            extract_id: value.extract_id,
            sequence_run_id: value.sequence_run_id,
            sequence_sample_id: value.sequence_sample_id,
            sequence_analysis_id: value.sequence_analysis_id,
            notes: value.notes,
            context: value.context,
            r#type: value.type_,
            file_type: value.file_type,
            url: value.url,
            licence: value.licence,
            access: value.access,
        }
    }
}
