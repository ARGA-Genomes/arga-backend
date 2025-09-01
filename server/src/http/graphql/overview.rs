use async_graphql::*;
use tracing::{instrument, info};

use super::helpers::ClassificationFilter;
use crate::database;
use crate::http::{Context as State, Error};

pub struct Overview;

#[Object]
impl Overview {
    #[instrument(skip(self, ctx), fields(classification_by = ?by))]
    async fn classification(&self, ctx: &Context<'_>, by: ClassificationFilter) -> Result<i64, Error> {
        info!("Fetching overview classification count");
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.classification(&by.into()).await?.total)
    }

    #[instrument(skip(self, ctx))]
    async fn sequences(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        info!("Fetching overview sequences count");
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.sequences().await?.total)
    }

    #[instrument(skip(self, ctx))]
    async fn loci(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        info!("Fetching overview loci count");
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.loci().await?.total)
    }

    #[instrument(skip(self, ctx))]
    async fn specimens(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        info!("Fetching overview specimens count");
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.specimens().await?.total)
    }

    /// Returns the amount of whole genomes in the index
    #[instrument(skip(self, ctx))]
    async fn whole_genomes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        info!("Fetching overview whole genomes count");
        let state = ctx.data::<State>()?;
        Ok(state.database.overview.whole_genomes().await?.total)
    }

    /// Returns the amount of species in every source
    #[instrument(skip(self, ctx))]
    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<OverviewItem>, Error> {
        info!("Fetching overview sources");
        let state = ctx.data::<State>()?;
        let stats = state.database.overview.sources().await?;
        let sources = stats.into_iter().map(|s| s.into()).collect();
        Ok(sources)
    }

    /// Returns the amount of species in every dataset
    #[instrument(skip(self, ctx))]
    async fn datasets(&self, ctx: &Context<'_>) -> Result<Vec<OverviewItem>, Error> {
        info!("Fetching overview datasets");
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
