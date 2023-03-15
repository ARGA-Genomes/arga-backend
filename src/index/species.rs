use async_graphql::SimpleObject;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub use super::Taxonomy;


/// The distribution of a species in a specific locality.
///
/// A specific species rank taxon can have zero or more distributions
/// associated with it. A distribution itself encapsulates the location,
/// the threat status, and any remarks or notes about the distribution.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Distribution {
    pub locality: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,

    pub threat_status: Option<String>,
    pub source: Option<String>,
}


/// Get information about a particular species.
///
/// Providers implementing this trait can retrieve detailed information
/// on a specific species.
#[async_trait]
pub trait GetSpecies {
    type Error;

    /// Get taxonomic information for a specific species.
    async fn taxonomy(&self, canonical_name: &str) -> Result<Taxonomy, Self::Error>;
    /// Get location and status details of a specific species.
    async fn distribution(&self, canonical_name: &str) -> Result<Vec<Distribution>, Self::Error>;
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct GenomicData {
    pub canonical_name: Option<String>,
    pub r#type: Option<String>,
    pub data_resource: Option<String>,
    pub recorded_by: Option<Vec<String>>,
    pub license: Option<String>,
    pub provenance: Option<String>,
    pub event_date: Option<String>,

    pub accession: Option<String>,
    pub accession_uri: Option<String>,
    pub refseq_category: Option<String>,

}

#[async_trait]
pub trait GetGenomicData {
    type Error;
    async fn genomic_data(&self, canonical_name: &str) -> Result<Vec<GenomicData>, Self::Error>;
}
