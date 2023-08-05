use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;

use super::common::{Page, SpeciesCard};
use super::common::Taxonomy;
use super::helpers::SpeciesHelper;

pub struct Order {
    pub order: String,
}

#[Object]
impl Order {
    /// Get taxonomic information for a specific order.
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.database.order.taxonomy(&self.order).await?;

        Ok(taxonomy)
    }

    /// Get a list of species with enriched data ideal for displaying as a card.
    /// The enriched data includes the taxonomy, species photos, and data summaries.
    async fn species(&self, ctx: &Context<'_>, page: i64) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let page = state.database.order.species(&self.order, page).await?;
        let cards = helper.cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }
}
