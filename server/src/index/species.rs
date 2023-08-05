use async_graphql::SimpleObject;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use super::specimen::SpecimenDetails;
use crate::{database::models::Name, http::graphql::common::Taxonomy};


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
    /// Get all taxonomic information linked to a list of names.
    async fn taxa(&self, name: &Vec<Name>) -> Result<Vec<Taxonomy>, Self::Error>;
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
    pub associated_sequences: Option<AssociatedSequences>
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssociatedSequences {
    #[serde(rename(deserialize = "sequenceID"))]
    pub sequence_id: String,
    pub genbank_accession: String,
    pub markercode: String,
    pub nucleotides: String
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


/// Get specimens of a specific species.
#[async_trait]
pub trait GetSpecimens {
    type Error;

    /// Get specimens related to the species taxon.
    async fn specimens(&self, name: &Name) -> Result<Vec<SpecimenDetails>, Self::Error>;
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


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct WholeGenome {
    pub id: String,
    pub r#type: Option<String>,
    pub data_resource: Option<String>,
    pub recorded_by: Option<Vec<String>>,
    pub license: Option<String>,
    pub provenance: Option<String>,
    pub event_date: Option<String>,
    pub occurrence_year: Option<Vec<String>>,
    pub other_catalog_numbers: Option<Vec<String>>,

    pub accession: Option<String>,
    pub accession_uri: Option<String>,
    pub refseq_category: Option<String>,
    pub coordinates: Option<GeoCoordinates>,

    pub ncbi_nuccore: Option<String>,
    pub ncbi_bioproject: Option<String>,
    pub ncbi_biosample: Option<String>,
    pub mixs_0000005: Option<String>,
    pub mixs_0000029: Option<String>,
    pub mixs_0000026: Option<String>,

    pub paired_asm_comp: Option<String>,

    pub raw_recorded_by: Option<String>,
    pub ncbi_release_type: Option<String>,

    pub is_reference_sequence: bool,
}

#[async_trait]
pub trait GetWholeGenomes {
    type Error;
    async fn full_genomes(&self, names: &Vec<Name>) -> Result<Vec<WholeGenome>, Self::Error>;
    async fn partial_genomes(&self, names: &Vec<Name>) -> Result<Vec<WholeGenome>, Self::Error>;
    async fn reference_genomes(&self, names: &Vec<Name>) -> Result<Vec<WholeGenome>, Self::Error>;
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct TraceFile {
    pub id: String,
    pub metadata: serde_json::Value,

    pub peak_locations_user: Option<Vec<i32>>,
    pub peak_locations_basecaller: Option<Vec<i32>>,
    pub quality_values_user: Option<Vec<i32>>,
    pub quality_values_basecaller: Option<Vec<i32>>,
    pub sequences_user: Option<Vec<i32>>,
    pub sequences_basecaller: Option<Vec<i32>>,

    pub measurements_voltage: Option<Vec<i32>>,
    pub measurements_current: Option<Vec<i32>>,
    pub measurements_power: Option<Vec<i32>>,
    pub measurements_temperature: Option<Vec<i32>>,

    pub analyzed_g: Option<Vec<i32>>,
    pub analyzed_a: Option<Vec<i32>>,
    pub analyzed_t: Option<Vec<i32>>,
    pub analyzed_c: Option<Vec<i32>>,

    pub raw_g: Option<Vec<i32>>,
    pub raw_a: Option<Vec<i32>>,
    pub raw_t: Option<Vec<i32>>,
    pub raw_c: Option<Vec<i32>>,
}

#[async_trait]
pub trait GetTraceFiles {
    type Error;
    async fn trace_files(&self, names: &Vec<Name>) -> Result<Vec<TraceFile>, Self::Error>;
}
