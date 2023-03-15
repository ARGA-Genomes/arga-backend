use async_graphql::*;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::genus::{GetGenus, Taxonomy};


pub struct Genus {
    pub genus: String,
}

#[Object]
impl Genus {
    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.genus).await.unwrap();

        Ok(taxonomy)
    }
}
