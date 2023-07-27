use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;

use super::common::{Page, SpeciesCard};
use super::common::Taxonomy;
use super::helpers::SpeciesHelper;


pub struct Class {
    pub class: String,
}

#[Object]
impl Class {
    /// Get taxonomic information for a specific class.
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.database.class.taxonomy(&self.class).await?;

        Ok(taxonomy)
    }

    /// Get a list of species with enriched data ideal for displaying as a card.
    /// The enriched data includes the taxonomy, species photos, and data summaries.
    async fn species(&self, ctx: &Context<'_>, page: i64) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let page = state.database.class.species(&self.class, page).await?;
        let cards = helper.cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }
}