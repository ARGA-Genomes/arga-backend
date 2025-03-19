use arga_core::models;
use async_graphql::{SimpleObject, *};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{DatasetDetails, FilterItem, Page, SpeciesCard, convert_filters};
use super::helpers::SpeciesHelper;
use super::taxon::{DataBreakdown, TaxonSummary};
use crate::database::Database;
use crate::database::extensions::classification_filters::Classification;
use crate::database::extensions::filters::Filter;
use crate::database::extensions::species_filters::NameAttributeFilter;
use crate::http::graphql::common::datasets::{AccessRightsStatus, DataReuseStatus, SourceContentType};
use crate::http::{Context as State, Error};

#[derive(OneofObject)]
pub enum SourceBy {
    Id(Uuid),
    Name(String),
}

#[derive(MergedObject)]
pub struct Source(SourceDetails, SourceQuery);

impl Source {
    pub async fn new(
        db: &Database,
        by: &SourceBy,
        filters: Vec<FilterItem>,
        species_attribute: Option<NameAttributeFilter>,
    ) -> Result<Source, Error> {
        let source = match by {
            SourceBy::Id(id) => db.sources.find_by_id(id).await?,
            SourceBy::Name(name) => db.sources.find_by_name(name).await?,
        };
        let details = source.clone().into();
        let query = SourceQuery {
            source,
            filters: convert_filters(filters)?,
            species_attribute,
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
                    species_attribute: None,
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
    species_attribute: Option<NameAttributeFilter>,
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
            .species(&self.source, &self.filters, page, page_size, &self.species_attribute)
            .await?;

        let cards = helper.filtered_cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }

    async fn summary(&self, ctx: &Context<'_>) -> Result<TaxonSummary, Error> {
        let state = ctx.data::<State>()?;
        let summary = state
            .database
            .taxa
            .taxon_summary(&Classification::Domain("Eukaryota".to_string()), &self.species_attribute)
            .await?;

        Ok(summary.into())
    }

    async fn species_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>()?;
        let summaries = state
            .database
            .taxa
            .species_summary(&Classification::Domain("Eukaryota".to_string()), &self.species_attribute)
            .await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();

        Ok(summaries)
    }

    async fn species_genome_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>()?;
        let summaries = state
            .database
            .taxa
            .species_genome_summary(&Classification::Domain("Eukaryota".to_string()), &self.species_attribute)
            .await?;

        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
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
    pub lists_id: Option<String>,
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
            lists_id: value.lists_id,
            reuse_pill: value.reuse_pill.map(|r| r.into()),
            access_pill: value.access_pill.map(|a| a.into()),
            content_type: value.content_type.map(|c| c.into()),
        }
    }
}
