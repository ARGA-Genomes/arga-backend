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

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItem {
    id: String,

    /// The scientific name given to this taxon
    scientific_name: Option<String>,
    /// The taxonomic genus
    genus: Option<String>,
    /// The taxonomic sub genus
    subgenus: Option<String>,
    /// The taxonomic kingdom
    kingdom: Option<String>,
    /// The taxonomic phylum
    phylum: Option<String>,
    /// The taxonomic family
    family: Option<String>,
    /// The taxonomic class
    class: Option<String>,

    species_group: Option<Vec<String>>,
    species_subgroup: Option<Vec<String>>,
    biome: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,
    license: Option<String>,

    recorded_by: Option<Vec<String>>,
    identified_by: Option<Vec<String>>,
}


#[derive(Clone, Debug)]
pub struct SearchFilterItem {
    pub field: String,
    pub value: String,
}

#[async_trait]
pub trait Searchable {
    type Error;

    async fn filtered(&self, filters: &Vec<SearchFilterItem>) -> Result<SearchResults, Self::Error>;
}
