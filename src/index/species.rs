use async_graphql::SimpleObject;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;


/// Taxonomic information of a species.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Taxonomy {
    /// The species name without authors.
    pub canonical_name: Option<String>,
    /// The species name author.
    pub authorship: Option<String>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
}


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
pub trait Species {
    type Error;

    /// Get taxonomic information for a specific species.
    async fn taxonomy(&self, taxon_uuid: Uuid) -> Result<Taxonomy, Self::Error>;
    /// Get location and status details of a specific species.
    async fn distribution(&self, taxon_uuid: Uuid) -> Result<Vec<Distribution>, Self::Error>;
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Specimen {
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
pub trait Specimens {
    type Error;

    async fn specimens_by_canonical_name(&self, canonical_name: &str) -> Result<Vec<Specimen>, Self::Error>;
}
