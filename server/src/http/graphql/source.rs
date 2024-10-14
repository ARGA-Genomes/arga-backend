use arga_core::models;
use async_graphql::SimpleObject;
use async_graphql::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{convert_filters, DatasetDetails, FilterItem, Page, SpeciesCard};
use super::helpers::SpeciesHelper;
use crate::database::extensions::filters::Filter;
use crate::database::Database;
use crate::http::graphql::common::datasets::AccessRightsStatus;
use crate::http::graphql::common::datasets::DataReuseStatus;
use crate::http::graphql::common::datasets::SourceContentType;
use crate::http::{Context as State, Error};

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
        let query = SourceQuery {
            source,
            filters: convert_filters(filters)?,
        };
        Ok(Source(details, query))
    }

    pub async fn all(db: &Database) -> Result<Vec<Source>, Error> {
        let records = db.sources.all_records().await?;
        let sources = records
            .into_iter()
            .map(|record| {
                let details = SourceDetails::from(record.clone());
                let query = SourceQuery {
                    source: record,
                    filters: vec![],
                };
                Source(details, query)
            })
            .collect();
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
        let state = ctx.data::<State>()?;
        let records = state.database.sources.datasets(&self.source).await?;
        let datasets = records.into_iter().map(|dataset| dataset.into()).collect();
        Ok(datasets)
    }

    async fn species(&self, ctx: &Context<'_>, page: i64, page_size: i64) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>()?;
        let helper = SpeciesHelper::new(&state.database);

        let page = state
            .database
            .sources
            .species(&self.source, &self.filters, page, page_size)
            .await?;
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
    pub reuse_pill: Option<DataReuseStatus>,
    pub access_pill: Option<AccessRightsStatus>,
    pub content_type: Option<SourceContentType>,
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
            reuse_pill: value.reuse_pill.map(|r| r.into()),
            access_pill: value.access_pill.map(|a| a.into()),
            content_type: value.content_type.map(|c| c.into()),
        }
    }
}
