use async_graphql::SimpleObject;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use super::providers::db::models::{NameList, Name, TaxonPhoto, UserTaxon};


#[async_trait]
pub trait GetListNames {
    type Error;
    async fn list_names(&self, list: &NameList) -> Result<Vec<Name>, Self::Error>;
}

#[async_trait]
pub trait GetListTaxa {
    type Error;
    async fn list_taxa(&self, names: &Vec<Name>) -> Result<Vec<UserTaxon>, Self::Error>;
}

#[async_trait]
pub trait GetListPhotos {
    type Error;
    async fn list_photos(&self, names: &Vec<Name>) -> Result<Vec<TaxonPhoto>, Self::Error>;
}


#[derive(Clone, Debug, Serialize, Deserialize, Default, SimpleObject)]
pub struct ListDataSummary {
    pub whole_genomes: usize,
    pub mitogenomes: usize,
    pub barcodes: usize,
    pub other: usize,
}

#[async_trait]
pub trait GetListDataSummary {
    type Error;
    async fn list_data_summary(&self, names: &Vec<Name>) -> Result<Vec<TaxonPhoto>, Self::Error>;
}
