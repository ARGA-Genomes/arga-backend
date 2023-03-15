use async_graphql::*;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::family::{GetFamily, Taxonomy};


pub struct Family {
    pub family: String,
}

#[Object]
impl Family {
    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.family).await.unwrap();

        Ok(taxonomy)
    }
}
