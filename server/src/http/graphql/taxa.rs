use async_graphql::*;

use crate::database::extensions::filters::Filter;
use crate::http::Error;
use crate::http::Context as State;

use super::common::{Page, SpeciesCard, FilterItem, convert_filters};
use super::helpers::SpeciesHelper;


pub struct Taxa {
    filters: Vec<Filter>,
}

#[Object]
impl Taxa {
    #[graphql(skip)]
    pub fn new(filters: Vec<FilterItem>) -> Result<Taxa, Error> {
        Ok(Taxa {
            filters: convert_filters(filters)?,
        })
    }

    async fn species(&self, ctx: &Context<'_>, page: i64, per_page: i64) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let page = state.database.taxa.species(&self.filters, page, per_page).await?;
        let cards = helper.cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }
}
