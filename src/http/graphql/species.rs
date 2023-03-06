use async_graphql::*;
use uuid::Uuid;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::species::{Taxonomy, Distribution, Specimen};
use crate::index::species::{Species as SpeciesTrait, Specimens};


pub struct Species {
    pub taxon_uuid: Uuid,
}

#[Object]
impl Species {
    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(self.taxon_uuid).await.unwrap();

        Ok(taxonomy)
    }

    #[instrument(skip(self, ctx))]
    async fn distribution(&self, ctx: &Context<'_>) -> Result<Vec<Distribution>, Error> {
        let state = ctx.data::<State>().unwrap();
        let distribution = state.db_provider.distribution(self.taxon_uuid).await.unwrap();

        Ok(distribution)
    }

    #[instrument(skip(self, ctx))]
    async fn specimens(&self, ctx: &Context<'_>) -> Result<Vec<Specimen>, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(self.taxon_uuid).await.unwrap();

        let specimens = if let Some(canonical_name) = taxonomy.canonical_name {
            state.provider.specimens_by_canonical_name(&canonical_name).await?
        } else {
            vec![]
        };

        Ok(specimens)
    }
}
