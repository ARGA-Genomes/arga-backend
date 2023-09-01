use arga_core::models;
use async_graphql::*;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;
use uuid::Uuid;

use crate::http::Error;
use crate::http::Context as State;
use crate::database::Database;

use super::common::{Page, SpeciesCard};
use super::helpers::SpeciesHelper;


#[derive(MergedObject)]
pub struct Dataset(DatasetDetails, DatasetQuery);

impl Dataset {
    pub async fn new(db: &Database, name: &str) -> Result<Dataset, Error> {
        let dataset = db.datasets.find_by_name(name).await?;
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


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct DatasetDetails {
    pub id: Uuid,
    pub global_id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub citation: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<models::Dataset> for DatasetDetails {
    fn from(value: models::Dataset) -> Self {
        Self {
            id: value.id,
            global_id: value.global_id,
            name: value.name,
            short_name: value.short_name,
            description: value.description,
            url: value.url,
            citation: value.citation,
            license: value.license,
            rights_holder: value.rights_holder,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
