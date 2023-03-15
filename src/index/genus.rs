use async_trait::async_trait;

pub use super::Taxonomy;


/// Get information about a particular genus.
///
/// Providers implementing this trait can retrieve detailed information
/// on a specific genus.
#[async_trait]
pub trait GetGenus {
    type Error;

    /// Get taxonomic information for a specific genus.
    async fn taxonomy(&self, canonical_name: &str) -> Result<Taxonomy, Self::Error>;
}
