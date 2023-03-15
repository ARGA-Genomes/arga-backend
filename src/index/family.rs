use async_trait::async_trait;

pub use super::Taxonomy;


/// Get information about a particular family.
///
/// Providers implementing this trait can retrieve detailed information
/// on a specific family.
#[async_trait]
pub trait GetFamily {
    type Error;

    /// Get taxonomic information for a specific family.
    async fn taxonomy(&self, canonical_name: &str) -> Result<Taxonomy, Self::Error>;
}
