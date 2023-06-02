use async_graphql::SimpleObject;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub use super::Taxonomy;
use super::providers::db::models::Name;


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
    async fn taxonomy(&self, name: &Name) -> Result<Taxonomy, Self::Error>;
    /// Get location and status details of a specific species.
    async fn distribution(&self, canonical_name: &str) -> Result<Vec<Distribution>, Self::Error>;
}


/// A region that a species inhabit.
///
/// Regions are less granular than a distribution and serves to more
/// clearly identify geographic locations inhabited by a particular species.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, SimpleObject)]
pub struct Region {
    pub name: String,
}

/// Get region information about a particular species.
///
/// Providers implementing this trait can retrieve detailed information
/// about where a species geographically inhabit.
#[async_trait]
pub trait GetRegions {
    type Error;

    /// Get the IBRA regions for the specified species.
    async fn ibra(&self, name: &Name) -> Result<Vec<Region>, Self::Error>;

    /// Get the IMCRA regions for the specified species.
    async fn imcra(&self, name: &Name) -> Result<Vec<Region>, Self::Error>;
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
    pub coordinates: Option<GeoCoordinates>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct GeoCoordinates {
    pub latitude: f32,
    pub longitude: f32,
}

#[async_trait]
pub trait GetGenomicData {
    type Error;
    async fn genomic_data(&self, canonical_name: &str) -> Result<Vec<GenomicData>, Self::Error>;
}


/// A photo of a verified species.
///
/// Photos are either links to external sources or from our own saved
/// store. Either way they should all have a license and attribution.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, SimpleObject)]
pub struct Photo {
    pub url: String,
    pub publisher: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub reference_url: Option<String>,
}

/// Get media metadata for a specific taxon.
#[async_trait]
pub trait GetMedia {
    type Error;

    /// Get media photos assigned to the species taxon.
    async fn photos(&self, name: &Name) -> Result<Vec<Photo>, Self::Error>;
}


/// A specimen of a specific species.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Specimen {
    pub type_status: String,
    pub institution_name: Option<String>,
    pub organism_id: Option<String>,
    pub locality: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub details: Option<String>,
    pub remarks: Option<String>,
}

/// Get specimens of a specific species.
#[async_trait]
pub trait GetSpecimens {
    type Error;

    /// Get specimens related to the species taxon.
    async fn specimens(&self, name: &Name) -> Result<Vec<Specimen>, Self::Error>;
}


/// Conservation statuses of a specific species.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct ConservationStatus {
    pub status: String,
    pub state: Option<String>,
    pub source: Option<String>,
}

/// Get the conservation status of a specific species.
#[async_trait]
pub trait GetConservationStatus {
    type Error;

    /// Get all conservation statuses assigned to the species.
    async fn conservation_status(&self, name: &Name) -> Result<Vec<ConservationStatus>, Self::Error>;
}
