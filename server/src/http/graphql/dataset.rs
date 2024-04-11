use arga_core::models;
use async_graphql::*;
use tracing::instrument;
use uuid::Uuid;

use super::common::datasets::DatasetDetails;
use super::common::{Page, SpeciesCard};
use super::helpers::SpeciesHelper;
use crate::database::Database;
use crate::http::{Context as State, Error};

#[derive(OneofObject)]
pub enum DatasetBy {
    Id(Uuid),
    Name(String),
}

#[derive(MergedObject)]
pub struct Dataset(DatasetDetails, DatasetQuery);

impl Dataset {
    pub async fn new(db: &Database, by: &DatasetBy) -> Result<Dataset, Error> {
        let dataset = match by {
            DatasetBy::Id(id) => db.datasets.find_by_id(id).await?,
            DatasetBy::Name(name) => db.datasets.find_by_name(name).await?,
        };
        let details = dataset.clone().into();
        let query = DatasetQuery { dataset };
        Ok(Dataset(details, query))
    }
}

pub struct DatasetQuery {
    dataset: models::Dataset,
}

#[Object]
impl DatasetQuery {
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
