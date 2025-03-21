use async_graphql::*;

use super::helpers::ClassificationFilter;
use crate::database;
use crate::http::{Context as State, Error};

pub struct Overview;

#[Object]
impl Overview {
    async fn classification(&self, ctx: &Context<'_>, by: ClassificationFilter) -> Result<i64, Error> {
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.classification(&by.into()).await?.total)
    }

    async fn sequences(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.sequences().await?.total)
    }

    async fn loci(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.loci().await?.total)
    }

    async fn specimens(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.specimens().await?.total)
    }

    /// Returns the amount of whole genomes in the index
    async fn whole_genomes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.whole_genomes().await?.total)
    }

    /// Returns the amount of species in every source
    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<OverviewItem>, Error> {
        let state = ctx.data::<State>()?;
        let stats = state.database.overview.sources().await?;
        let sources = stats.into_iter().map(|s| s.into()).collect();
        Ok(sources)
    }

    /// Returns the amount of species in every dataset
    async fn datasets(&self, ctx: &Context<'_>) -> Result<Vec<OverviewItem>, Error> {
        let state = ctx.data::<State>()?;
        let stats = state.database.overview.datasets().await?;
        let sources = stats.into_iter().map(|s| s.into()).collect();
        Ok(sources)
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct OverviewItem {
    name: String,
    total: i64,
}

impl From<database::overview::OverviewRecord> for OverviewItem {
    fn from(value: database::overview::OverviewRecord) -> Self {
        Self {
            name: value.name,
            total: value.total,
        }
    }
}
