use arga_core::models;
use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::database::extensions::filters::Filter;
use crate::http::Error;
use crate::http::Context as State;
use crate::database::Database;

use super::common::{Page, SpeciesCard, FilterItem, convert_filters};
use super::dataset::DatasetDetails;
use super::helpers::SpeciesHelper;


#[derive(OneofObject)]
pub enum SourceBy {
    Id(Uuid),
    Name(String),
}

#[derive(MergedObject)]
pub struct Source(SourceDetails, SourceQuery);

impl Source {
    pub async fn new(db: &Database, by: &SourceBy, filters: Vec<FilterItem>) -> Result<Source, Error> {
        let source = match by {
            SourceBy::Id(id) => db.sources.find_by_id(id).await?,
            SourceBy::Name(name) => db.sources.find_by_name(name).await?,
        };
        let details = source.clone().into();
        let query = SourceQuery { source, filters: convert_filters(filters)? };
        Ok(Source(details, query))
    }

    pub async fn all(db: &Database) -> Result<Vec<SourceDetails>, Error> {
        let records = db.sources.all_records().await?;
        let sources = records.into_iter().map(|record| SourceDetails::from(record)).collect();
        Ok(sources)
    }
}


pub struct SourceQuery {
    source: models::Source,
    filters: Vec<Filter>,
}

#[Object]
impl SourceQuery {
    async fn datasets(&self, ctx: &Context<'_>) -> Result<Vec<DatasetDetails>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.database.sources.datasets(&self.source).await?;
        let datasets = records.into_iter().map(|dataset| dataset.into()).collect();
        Ok(datasets)
    }

    async fn species(&self, ctx: &Context<'_>, page: i64, page_size: i64) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let page = state.database.sources.species(&self.source, &self.filters, page, page_size).await?;
        let cards = helper.filtered_cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SourceDetails {
    pub id: Uuid,
    pub name: String,
    pub author: String,
    pub rights_holder: String,
    pub access_rights: String,
    pub license: String,
}

impl From<models::Source> for SourceDetails {
    fn from(value: models::Source) -> Self {
        Self {
            id: value.id,
            name: value.name,
            author: value.author,
            rights_holder: value.rights_holder,
            access_rights: value.access_rights,
            license: value.license,
        }
    }
}
