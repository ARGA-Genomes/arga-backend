use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::stats::GenusBreakdownItem;
use crate::index::stats::GetGenusBreakdown;
use crate::index::stats::GetGenusStats;


pub struct Statistics;

#[Object]
impl Statistics {
    #[instrument(skip(self, ctx))]
    async fn genus(&self, ctx: &Context<'_>, genus: String) -> Result<Statistic, Error> {
        let state = ctx.data::<State>().unwrap();
        let db_stats = state.db_provider.genus_stats(&genus).await.unwrap();
        let solr_stats = state.provider.genus_stats(&genus).await.unwrap();
        let solr_breakdown = state.provider.species_breakdown(&genus).await.unwrap();

        Ok(Statistic {
            total_species: db_stats.total_species,
            species_with_data: solr_stats.total_species,
            breakdown: solr_breakdown.species,
        })
    }
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistic {
    /// The total amount of species
    pub total_species: i64,
    /// The total amount of species that have data records
    pub species_with_data: i64,

    /// A breakdown of species and the amount of data for it
    pub breakdown: Vec<GenusBreakdownItem>,
}
