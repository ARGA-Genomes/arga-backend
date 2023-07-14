use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;

use super::common::SpeciesCard;
use super::helpers::SpeciesHelper;


pub struct Assemblies;

#[Object]
impl Assemblies {
    async fn species(&self, ctx: &Context<'_>) -> Result<Vec<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let taxa = state.database.assembly.species().await?;
        let cards = helper.cards(taxa).await?;
        Ok(cards)
    }
}
