use async_trait::async_trait;

use super::providers::db::models::{UserTaxaList, Name, TaxonPhoto, UserTaxon};


#[async_trait]
pub trait GetListNames {
    type Error;
    async fn list_names(&self, list: &UserTaxaList) -> Result<Vec<Name>, Self::Error>;
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
