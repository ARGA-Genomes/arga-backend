use async_graphql::*;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::species::{Taxonomy, Distribution, GenomicData};
use crate::index::species::{GetSpecies, GetGenomicData};


pub struct Species {
    pub canonical_name: String,
}

#[Object]
impl Species {
    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.canonical_name).await.unwrap();

        Ok(taxonomy)
    }

    #[instrument(skip(self, ctx))]
    async fn distribution(&self, ctx: &Context<'_>) -> Result<Vec<Distribution>, Error> {
        let state = ctx.data::<State>().unwrap();
        let distribution = state.db_provider.distribution(&self.canonical_name).await.unwrap();

        Ok(distribution)
    }

    #[instrument(skip(self, ctx))]
    async fn data(&self, ctx: &Context<'_>) -> Result<Vec<GenomicData>, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.canonical_name).await.unwrap();

        let data = if let Some(canonical_name) = taxonomy.canonical_name {
            state.provider.genomic_data(&canonical_name).await?
        } else {
            vec![]
        };

        Ok(data)
    }
}
