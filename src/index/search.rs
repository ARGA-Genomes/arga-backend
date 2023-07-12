use axum::async_trait;
use async_graphql::{SimpleObject, Union, Enum};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::database::models::{ArgaTaxon};
use crate::http::graphql::lists::SpeciesPhoto;
use crate::http::graphql::search::WithRecordType;
use crate::index::lists::{ListDataSummary, Pagination};


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

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchItem {
    pub id: String,
    pub species_uuid: Option<String>,

    pub genomic_data_records: Option<usize>,

    /// The scientific name given to this species
    pub scientific_name: Option<String>,

    /// The scientific name without authorship
    pub canonical_name: Option<String>,

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
    pub method: SearchFilterMethod,
}

#[derive(Clone, Debug)]
pub enum SearchFilterMethod {
    Include,
    Exclude,
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
}

/// Free text search for a species dataset.
///
/// Providers implementing this trait allow searching a species dataset
/// based on their taxa. The order and specific algorithm used for the search
/// is dependent on the provider.
#[async_trait]
pub trait TaxaSearch {
    type Error;

    /// Return search suggestions for autocomplete features.
    async fn suggestions(&self, query: &str) -> Result<Vec<SearchSuggestion>, Self::Error>;
}


#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesSearchItem {
    pub scientific_name: Option<String>,
    pub canonical_name: Option<String>,
    pub total_records: usize,
    pub total_genomic_records: Option<usize>,
    pub data_summary: ListDataSummary,
    pub photo: Option<SpeciesPhoto>,
}

#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesSearchResult {
    pub records: Vec<SpeciesSearchItem>,
}

#[async_trait]
pub trait SpeciesSearch {
    type Error;
    async fn search_species(&self, query: Option<String>, filters: &Vec<SearchFilterItem>, results_type: Option<WithRecordType>, pagination: Option<Pagination>) -> Result<SpeciesSearchResult, Self::Error>;
}

#[async_trait]
pub trait SpeciesSearchByCanonicalName {
    type Error;
    async fn search_species_by_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Self::Error>;
}

#[async_trait]
pub trait SpeciesSearchExcludingCanonicalName {
    type Error;
    async fn search_species_excluding_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Self::Error>;
}

#[async_trait]
pub trait SpeciesSearchWithRegion {
    type Error;
    async fn search_species_with_region(
        &self,
        region: &Vec<String>,
        filters: &Vec<SearchFilterItem>,
        offset: i64,
        limit: i64
    ) -> Result<Vec<ArgaTaxon>, Self::Error>;
}

#[async_trait]
pub trait DNASearchByCanonicalName {
    type Error;
    async fn search_dna_by_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Self::Error>;
}


#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct GenusSearchItem {
    pub genus_name: String,
    pub total_records: usize,
}

#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct GenusSearchResult {
    pub records: Vec<GenusSearchItem>,
}

#[async_trait]
pub trait GenusSearch {
    type Error;
    async fn search_genus(&self, query: &str, filters: &Vec<SearchFilterItem>) -> Result<GenusSearchResult, Self::Error>;
}


#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Enum)]
pub enum FullTextType {
    Taxon,
    ReferenceGenomeSequence,
    WholeGenomeSequence,
    PartialGenomeSequence,
    UnknownGenomeSequence,
    Barcode,
}

#[derive(Debug, Default, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Classification {
    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
}

#[derive(Debug, Default, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct AssemblySummary {
    pub whole_genomes: usize,
    pub partial_genomes: usize,
    pub reference_genomes: usize,
    pub barcodes: usize,
}

#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct TaxonItem {
    #[graphql(skip)]
    pub name_id: Uuid,
    pub scientific_name: String,
    pub scientific_name_authorship: Option<String>,
    pub canonical_name: Option<String>,
    pub rank: Option<String>,
    pub taxonomic_status: Option<String>,
    pub common_names: Vec<String>,
    pub classification: Classification,
    pub assembly_summary: AssemblySummary,
    pub score: f32,
    pub r#type: FullTextType,
}

#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct  GenomeSequenceItem {
    pub scientific_name: String,
    pub sequences: usize,
    pub score: f32,
    pub r#type: FullTextType,
}


#[derive(Debug, Union, Deserialize)]
pub enum FullTextSearchItem {
    Taxon(TaxonItem),
    GenomeSequence(GenomeSequenceItem)
}


#[derive(Debug, Deserialize, SimpleObject, Default)]
#[serde(rename_all = "camelCase")]
pub struct FullTextSearchResult {
    pub records: Vec<FullTextSearchItem>,
}

#[async_trait]
pub trait FullTextSearch {
    type Error;
    async fn full_text(&self, query: &str) -> Result<FullTextSearchResult, Self::Error>;
}


impl PartialOrd for FullTextSearchItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let score = match self {
            FullTextSearchItem::Taxon(item) => item.score,
            FullTextSearchItem::GenomeSequence(item) => item.score,
        };

        match other {
            FullTextSearchItem::Taxon(item) => score.partial_cmp(&item.score),
            FullTextSearchItem::GenomeSequence(item) => score.partial_cmp(&item.score),
        }
    }
}


impl PartialEq for FullTextSearchItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Taxon(l0), Self::Taxon(r0)) => l0.score == r0.score,
            (Self::GenomeSequence(l0), Self::GenomeSequence(r0)) => l0.score == r0.score,
            _ => false,
        }
    }
}
