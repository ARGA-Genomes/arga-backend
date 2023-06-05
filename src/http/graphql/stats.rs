use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::names::GetNames;
use crate::index::stats::{FamilyBreakdownItem, GenusBreakdownItem, GetFamilyBreakdown, GetFamilyStats, GetGenusBreakdown, GetGenusStats, GetSpeciesStats};


pub struct Statistics;

#[Object]
impl Statistics {
    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<SpeciesStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let names = state.db_provider.find_by_canonical_name(&canonical_name).await?;

        if names.is_empty() {
            return Err(Error::NotFound(canonical_name));
        }

        let solr_stats = state.provider.species_stats(&names).await?;

        // combine the stats for all species matching the canonical name
        let mut stats = SpeciesStatistics::default();
        for stat in solr_stats {
            stats.total += stat.total;
            stats.whole_genomes += stat.whole_genomes;
            stats.mitogenomes += stat.mitogenomes;
            stats.barcodes += stat.barcodes;
        }

        Ok(stats)
    }

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


#[derive(Clone, Debug, Default, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesStatistics {
    /// The total amount of genomic data
    pub total: usize,
    /// The total amount of whole genomes available
    pub whole_genomes: usize,
    /// The total amount of mitogenomes available
    pub mitogenomes: usize,
    /// The total amount of barcodes available
    pub barcodes: usize,
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
