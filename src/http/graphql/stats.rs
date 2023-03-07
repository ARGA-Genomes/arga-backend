use async_graphql::*;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::stats::{GetGenusStats, GenusStats};


pub struct Statistics;

#[Object]
impl Statistics {
    #[instrument(skip(self, ctx))]
    async fn genus(&self, ctx: &Context<'_>, genus: String) -> Result<GenusStats, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.db_provider.genus_stats(&genus).await.unwrap();

        Ok(stats)
    }
}
