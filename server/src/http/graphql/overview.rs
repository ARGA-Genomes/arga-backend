use async_graphql::*;

use crate::database;
use crate::http::Error;
use crate::http::Context as State;


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

    async fn loci(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.loci().await?.total)
    }

    async fn specimens(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.specimens().await?.total)
    }


    /// Returns the amount of species records for plants in the index
    async fn plants(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.plants().await?.total)
    }

    /// Returns the amount of species records for fungi in the index
    async fn fungi(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.fungi().await?.total)
    }

    /// Returns the amount of species records for protista in the index
    async fn protista(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.protista().await?.total)
    }


    /// Returns the amount of whole genomes in the index
    async fn whole_genomes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.whole_genomes().await?.total)
    }

    /// Returns the amount of species
    async fn all_species(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.all_species().await?.total)
    }


    /// Returns the amount of species in every dataset
    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<SourceOverview>, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.overview.sources().await?;
        let sources = stats.into_iter().map(|s| s.into()).collect();
        Ok(sources)
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct SourceOverview {
    id: uuid::Uuid,
    name: String,
    total: i64,
}

impl From<database::overview::SourceOverview> for SourceOverview {
    fn from(value: database::overview::SourceOverview) -> Self {
        Self {
            id: value.id,
            name: value.name,
            total: value.total,
        }
    }
}
