use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use crate::database::stats::BreakdownItem;
use crate::http::Error;
use crate::http::Context as State;

use crate::index::names::GetNames;


pub struct Statistics;

#[Object]
impl Statistics {
    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<SpeciesStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let names = state.database.find_by_canonical_name(&canonical_name).await?;

        if names.is_empty() {
            return Err(Error::NotFound(canonical_name));
        }

        let names = names.iter().map(|name| name.id).collect();
        let assembly_summaries = state.database.species.assembly_summary(&names).await?;
        let marker_summaries = state.database.species.marker_summary(&names).await?;

        // combine the stats for all species matching the canonical name
        let mut stats = SpeciesStatistics::default();
        for stat in assembly_summaries {
            stats.total += (stat.whole_genomes + stat.reference_genomes + stat.partial_genomes) as usize;
            stats.whole_genomes += stat.whole_genomes as usize;
            stats.partial_genomes += stat.partial_genomes as usize;
        }
        for stat in marker_summaries {
            stats.total += stat.barcodes as usize;
            stats.barcodes += stat.barcodes as usize;
        }

        Ok(stats)
    }

    #[instrument(skip(self, ctx))]
    async fn genus(&self, ctx: &Context<'_>, genus: String) -> Result<GenusStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.stats.genus(&genus).await?;
        let breakdown = state.database.stats.genus_breakdown(&genus).await?;

        Ok(GenusStatistics {
            total_species: stats.total_species,
            total_valid_species: stats.total_valid_species,
            species_with_data: breakdown.species.len(),
            breakdown: breakdown.species,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn family(&self, ctx: &Context<'_>, family: String) -> Result<FamilyStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.stats.family(&family).await?;
        let breakdown = state.database.stats.family_breakdown(&family).await?;

        Ok(FamilyStatistics {
            total_genera: stats.total_genera,
            genera_with_data: stats.total_genera_with_data,
            breakdown: breakdown.genera,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn order(&self, ctx: &Context<'_>, order: String) -> Result<OrderStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.stats.order(&order).await?;
        let breakdown = state.database.stats.order_breakdown(&order).await?;

        Ok(OrderStatistics {
            total_families: stats.total_families,
            families_with_data: stats.total_families_with_data,
            breakdown: breakdown.families,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn class(&self, ctx: &Context<'_>, class: String) -> Result<ClassStatistics, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.stats.class(&class).await?;
        let breakdown = state.database.stats.class_breakdown(&class).await?;

        Ok(ClassStatistics {
            total_orders: stats.total_orders,
            orders_with_data: stats.total_orders_with_data,
            breakdown: breakdown.orders,
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
pub struct GenusStatistics {
    /// The total amount of species
    pub total_species: usize,
    /// The total amount of valid species
    pub total_valid_species: usize,
    /// The total amount of species that have data records
    pub species_with_data: usize,

    /// A breakdown of species and the amount of data for it
    pub breakdown: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyStatistics {
    /// The total amount of genera
    pub total_genera: usize,
    /// The total amount of genera that have data records
    pub genera_with_data: usize,

    /// A breakdown of genera and the amount of data for it
    pub breakdown: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatistics {
    /// The total amount of families
    pub total_families: usize,
    /// The total amount of families that have data records
    pub families_with_data: usize,

    /// A breakdown of families and the amount of data for it
    pub breakdown: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStatistics {
    /// The total amount of orders
    pub total_orders: usize,
    /// The total amount of orders that have data records
    pub orders_with_data: usize,

    /// A breakdown of orders and the amount of data for it
    pub breakdown: Vec<BreakdownItem>,
}
