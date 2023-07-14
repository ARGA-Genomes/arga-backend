use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;

use super::common::SpeciesCard;
use super::common::Taxonomy;
use super::helpers::SpeciesHelper;


pub struct Genus {
    pub genus: String,
}

#[Object]
impl Genus {
    /// Get taxonomic information for a specific genus.
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.database.genus.taxonomy(&self.genus).await.unwrap();

        Ok(taxonomy)
    }

    /// Get a list of species with enriched data ideal for displaying as a card.
    /// The enriched data includes the taxonomy, species photos, and data summaries.
    async fn species(&self, ctx: &Context<'_>) -> Result<Vec<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let taxa = state.database.class.species(&self.genus).await?;
        let cards = helper.cards(taxa).await?;
        Ok(cards)
    }
}
