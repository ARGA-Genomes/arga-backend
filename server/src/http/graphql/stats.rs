use async_graphql::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::common::taxonomy::TaxonomicRank;
use crate::database::stats::BreakdownItem;
use crate::http::{Context as State, Error};


pub struct Statistics;

#[Object]
impl Statistics {
    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<SpeciesStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let names = state.database.names.find_by_canonical_name(&canonical_name).await?;

        if names.is_empty() {
            return Err(Error::NotFound(canonical_name));
        }

        //FIXME: endpoint no longer needed?
        // let names = names.iter().map(|name| name.id).collect();
        // let assembly_summaries = state.database.species.assembly_summary(&names).await?;
        // let marker_summaries = state.database.species.marker_summary(&names).await?;

        // combine the stats for all species matching the canonical name
        let mut stats = SpeciesStatistics::default();
        // for stat in assembly_summaries {
        //     stats.total += (stat.whole_genomes + stat.reference_genomes + stat.partial_genomes) as usize;
        //     stats.whole_genomes += stat.whole_genomes as usize;
        //     stats.partial_genomes += stat.partial_genomes as usize;
        // }
        // for stat in marker_summaries {
        //     stats.total += stat.barcodes as usize;
        //     stats.barcodes += stat.barcodes as usize;
        // }

        Ok(stats)
    }

    async fn rank_breakdown(&self, ctx: &Context<'_>, rank: TaxonomicRank) -> Result<RankStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let tree = state.database.stats.taxon_tree(rank.into()).await?;
        Ok(RankStatistics::default())
    }

    #[instrument(skip(self, ctx))]
    async fn dataset(&self, ctx: &Context<'_>, name: String) -> Result<DatasetStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.stats.dataset(&name).await?;
        let breakdown = state.database.stats.dataset_breakdown(&name).await?;

        Ok(DatasetStatistics {
            total_species: stats.total_species,
            species_with_data: stats.total_species_with_data,
            breakdown: breakdown.species,
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
    /// The total amount of partial genomes available
    pub partial_genomes: usize,
    /// The total amount of organelles available
    pub organelles: usize,
    /// The total amount of barcodes available
    pub barcodes: usize,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetStatistics {
    /// The total amount of species
    pub total_species: usize,
    /// The total amount of species that have data records
    pub species_with_data: usize,

    /// A breakdown of species and the amount of data for it
    pub breakdown: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, Default, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankStatistics {
    /// The total amount of genomic data
    pub total: usize,
    /// The total amount of whole genomes available
    pub whole_genomes: usize,
    /// The total amount of partial genomes available
    pub partial_genomes: usize,
    /// The total amount of organelles available
    pub organelles: usize,
    /// The total amount of barcodes available
    pub barcodes: usize,
}
