use async_graphql::{SimpleObject, InputObject};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::database::models::{NameList, Name, TaxonPhoto, UserTaxon};


#[derive(Debug)]
pub enum Filter {
    Kingdom(String),
    Phylum(String),
}

#[derive(Debug)]
pub enum FilterItem {
    Include(Filter),
    Exclude(Filter),
}

#[derive(Debug, Default)]
pub struct Filters {
    pub items: Vec<FilterItem>,
}

#[derive(Debug, InputObject)]
pub struct Pagination {
    pub page: i64,
    pub page_size: i64,
}


#[async_trait]
pub trait GetListNames {
    type Error;
    async fn list_names(&self, list: &NameList, filters: &Filters, pagination: &Pagination) -> Result<Vec<Name>, Self::Error>;
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


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct ListStats {
    pub total_records: usize,
}

#[async_trait]
pub trait GetListStats {
    type Error;
    async fn list_stats(&self, list: &NameList, filters: &Filters) -> Result<ListStats, Self::Error>;
}
