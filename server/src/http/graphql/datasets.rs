use async_graphql::*;

use tracing::instrument;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::http::Error;
use crate::http::Context as State;
use crate::database::{schema, Database};
use crate::database::models::Dataset;

use super::common::{Page, SpeciesCard};
use super::helpers::SpeciesHelper;


pub struct Datasets {
    pub dataset: Dataset,
}

#[Object]
impl Datasets {
    #[graphql(skip)]
    pub async fn new(db: &Database, name: &str) -> Result<Datasets, Error> {
        use schema::datasets;
        let mut conn = db.pool.get().await?;

        let dataset = datasets::table
            .filter(datasets::name.eq(&name))
            .get_result::<Dataset>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = dataset {
            return Err(Error::NotFound(name.to_string()));
        }

        let dataset = dataset?;

        Ok(Datasets { dataset })
    }

    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>, page: i64) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let page = state.database.datasets.species(&self.dataset, page).await?;
        let cards = helper.cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }
}
