use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;
use crate::index::overview::{Overview as OverviewTrait, OverviewCategory};


pub struct Overview;

#[Object]
impl Overview {
    /// Returns the amount of genomic records for animals in the index
    async fn animals(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.animals().await?.total)
    }

    async fn sequences(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.sequences().await?.total)
    }

    async fn markers(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.markers().await?.total)
    }


    /// Returns the amount of genomic records for plants in the index
    async fn plants(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.plants().await?.total)
    }

    /// Returns the amount of genomic records for fungi in the index
    async fn fungi(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.fungi().await?.total)
    }


    /// Returns the amount of whole genomes in the index
    async fn whole_genomes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.whole_genomes().await?.total)
    }

    /// Returns the amount of records
    async fn all_records(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::AllRecords).await?)
    }

    /// Returns the amount of species
    async fn all_species(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.all_species().await?.total)
    }
}
