use async_trait::async_trait;
use async_graphql::SimpleObject;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenusStats {
    /// The total amount of species in the genus
    pub total_species: usize,
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


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenusBreakdown {
    pub species: Vec<GenusBreakdownItem>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenusBreakdownItem {
    pub name: String,
    pub total: usize,
}


#[async_trait]
pub trait GetGenusBreakdown {
    type Error;
    async fn genus_breakdown(&self, genus: &str) -> Result<GenusBreakdown, Self::Error>;
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyStats {
    /// The total amount of genera in the family
    pub total_genera: usize,
}

/// Gets stats for a specific family.
///
/// Providers implementing this trait will calculate both simple
/// and detailed statistics about a family.
#[async_trait]
pub trait GetFamilyStats {
    type Error;
    async fn family_stats(&self, genus: &str) -> Result<FamilyStats, Self::Error>;
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyBreakdown {
    pub genera: Vec<FamilyBreakdownItem>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyBreakdownItem {
    pub name: String,
    pub total: usize,
}


#[async_trait]
pub trait GetFamilyBreakdown {
    type Error;
    async fn family_breakdown(&self, family: &str) -> Result<FamilyBreakdown, Self::Error>;
}
