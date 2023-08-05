use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;

use super::common::{Page, SpeciesCard};
use super::helpers::SpeciesHelper;


pub struct Assemblies;

#[Object]
impl Assemblies {
    async fn species(&self, ctx: &Context<'_>, page: i64) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let page = state.database.assembly.species(page).await?;
        let taxa = helper.taxonomy(&page.records).await?;
        let cards = helper.cards(taxa).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }
}
