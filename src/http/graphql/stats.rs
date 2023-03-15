use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::stats::FamilyBreakdownItem;
use crate::index::stats::GenusBreakdownItem;
use crate::index::stats::GetFamilyBreakdown;
use crate::index::stats::GetFamilyStats;
use crate::index::stats::GetGenusBreakdown;
use crate::index::stats::GetGenusStats;


pub struct Statistics;

#[Object]
impl Statistics {
    #[instrument(skip(self, ctx))]
    async fn genus(&self, ctx: &Context<'_>, genus: String) -> Result<GenusStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let db_stats = state.db_provider.genus_stats(&genus).await.unwrap();
        let solr_stats = state.provider.genus_stats(&genus).await.unwrap();
        let solr_breakdown = state.provider.genus_breakdown(&genus).await.unwrap();

        Ok(GenusStatistics {
            total_species: db_stats.total_species,
            species_with_data: solr_stats.total_species,
            breakdown: solr_breakdown.species,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn family(&self, ctx: &Context<'_>, family: String) -> Result<FamilyStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let db_stats = state.db_provider.family_stats(&family).await.unwrap();
        let solr_stats = state.provider.family_stats(&family).await.unwrap();
        let solr_breakdown = state.provider.family_breakdown(&family).await.unwrap();

        Ok(FamilyStatistics {
            total_genera: db_stats.total_genera,
            genera_with_data: solr_stats.total_genera,
            breakdown: solr_breakdown.genera,
        })
    }
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenusStatistics {
    /// The total amount of species
    pub total_species: usize,
    /// The total amount of species that have data records
    pub species_with_data: usize,

    /// A breakdown of species and the amount of data for it
    pub breakdown: Vec<GenusBreakdownItem>,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyStatistics {
    /// The total amount of genera
    pub total_genera: usize,
    /// The total amount of genera that have data records
    pub genera_with_data: usize,

    /// A breakdown of genera and the amount of data for it
    pub breakdown: Vec<FamilyBreakdownItem>,
}
