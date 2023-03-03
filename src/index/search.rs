use axum::async_trait;
use async_graphql::SimpleObject;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SearchResults {
    /// The total amount of records matching the search
    pub total: usize,
    /// An array of records matching the search
    pub records: Vec<SearchItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesList {
    pub total: usize,
    pub groups: Vec<GroupedSearchItem>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItem {
    pub id: String,
    pub species_uuid: Option<String>,

    pub genomic_data_records: Option<usize>,

    /// The scientific name given to this taxon
    pub scientific_name: Option<String>,
    /// The taxonomic genus
    pub genus: Option<String>,
    /// The taxonomic sub genus
    pub subgenus: Option<String>,
    /// The taxonomic kingdom
    pub kingdom: Option<String>,
    /// The taxonomic phylum
    pub phylum: Option<String>,
    /// The taxonomic family
    pub family: Option<String>,
    /// The taxonomic class
    pub class: Option<String>,

    pub species: Option<String>,
    pub species_group: Option<Vec<String>>,
    pub species_subgroup: Option<Vec<String>>,
    pub biome: Option<String>,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub license: Option<String>,

    pub recorded_by: Option<Vec<String>>,
    pub identified_by: Option<Vec<String>>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedSearchItem {
    pub key: Option<String>,
    pub matches: usize,
    pub records: Vec<SearchItem>
}


#[derive(Clone, Debug)]
pub struct SearchFilterItem {
    pub field: String,
    pub value: String,
}


#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestion {
    pub guid: String,
    pub species_name: String,
    pub common_name: Option<String>,
    pub matched: String,
}


#[async_trait]
pub trait Searchable {
    type Error;

    async fn filtered(&self, filters: &Vec<SearchFilterItem>) -> Result<SearchResults, Self::Error>;

    async fn species(&self, filters: &Vec<SearchFilterItem>) -> Result<SpeciesList, Self::Error>;

    async fn suggestions(&self, query: &str) -> Result<Vec<SearchSuggestion>, Self::Error>;
}
