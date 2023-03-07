use async_trait::async_trait;
use async_graphql::SimpleObject;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenusStats {
    /// The total amount of species in the genus
    pub total_species: i64,
}

/// Gets stats for a specific genus.
///
/// Providers implementing this trait will calculate both simple
/// and detailed statistics about a genus.
#[async_trait]
pub trait GetGenusStats {
    type Error;
    async fn genus_stats(&self, genus: &str) -> Result<GenusStats, Self::Error>;
}
