use arga_core::models;
use async_graphql::{SimpleObject, *};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::species::{SortDirection, SpeciesSort};
use super::common::{DatasetDetails, FilterItem, Page, SpeciesCard, convert_filters};
use super::helpers::{self, SpeciesHelper, csv};
use super::taxon::{DataBreakdown, RankSummary};
use crate::database::extensions::filters::Filter;
use crate::database::extensions::species_filters::{self};
use crate::database::{Database, sources};
use crate::http::graphql::common::datasets::{AccessRightsStatus, DataReuseStatus, SourceContentType};
use crate::http::{Context as State, Error};

#[derive(OneofObject)]
pub enum SourceBy {
    Id(Uuid),
    Name(String),
}

#[derive(SimpleObject, Serialize)]
pub struct GenomeRelease {
    pub scientific_name: String,
    pub canonical_name: String,
    pub release_date: Option<NaiveDate>,
}

impl From<sources::GenomeRelease> for GenomeRelease {
    fn from(value: sources::GenomeRelease) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            release_date: value.release_date,
        }
    }
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

    async fn species(
        &self,
        ctx: &Context<'_>,
        page: i64,
        page_size: i64,
        sort: Option<SpeciesSort>,
        sort_direction: Option<SortDirection>,
    ) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>()?;
        let helper = SpeciesHelper::new(&state.database);

        let page = state
            .database
            .sources
            .species(
                &self.source,
                &self.filters,
                page,
                page_size,
                match sort {
                    Some(srt) => srt.into(),
                    _ => species_filters::SpeciesSort::ScientificName,
                },
                match sort_direction {
                    Some(dir) => dir.into(),
                    _ => species_filters::SortDirection::Asc,
                },
            )
            .await?;

        let cards = helper.filtered_cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }

    async fn species_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;

        let page = state
            .database
            .sources
            .species(
                &self.source,
                &self.filters,
                1,       // hard coded page size
                1000000, // some arbitrary number of records that hopefully is enough for all of them (1 million)
                species_filters::SpeciesSort::ScientificName,
                species_filters::SortDirection::Asc,
            )
            .await?;

        let csv = helpers::csv::species(page.records).await?;

        Ok(csv)
    }

    async fn species_genomic_data_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .species_genomic_data_summary(&self.source, &filters_option)
            .await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_genomic_data_summary_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .species_genomic_data_summary(&self.source, &filters_option)
            .await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();
        let csv = helpers::csv::generic(summaries).await?;
        Ok(csv)
    }

    async fn species_genomes_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .species_genomes_summary(&self.source, &filters_option)
            .await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_genomes_summary_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .species_genomes_summary(&self.source, &filters_option)
            .await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();
        let csv = helpers::csv::generic(summaries).await?;
        Ok(csv)
    }

    async fn species_loci_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .species_loci_summary(&self.source, &filters_option)
            .await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_loci_summary_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .species_loci_summary(&self.source, &filters_option)
            .await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();
        let csv = helpers::csv::generic(summaries).await?;
        Ok(csv)
    }

    async fn latest_genome_releases(&self, ctx: &Context<'_>) -> Result<Vec<GenomeRelease>, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .latest_genome_releases(&self.source, &filters_option)
            .await?;
        let summaries: Vec<GenomeRelease> = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn latest_genome_releases_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summaries = state
            .database
            .sources
            .latest_genome_releases(&self.source, &filters_option)
            .await?;
        let summaries: Vec<GenomeRelease> = summaries.into_iter().map(|r| r.into()).collect();
        let csv = helpers::csv::generic(summaries).await?;
        Ok(csv)
    }

    async fn summary(&self, ctx: &Context<'_>) -> Result<RankSummary, Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summary = state.database.sources.summary(&self.source, &filters_option).await?;
        Ok(summary.into())
    }

    async fn summary_csv(&self, ctx: &Context<'_>) -> Result<String, async_graphql::Error> {
        let state = ctx.data::<State>()?;
        let filters_option = (!self.filters.is_empty()).then(|| self.filters.clone());
        let summary = state.database.sources.summary(&self.source, &filters_option).await?;
        let out: Vec<RankSummary> = vec![summary.into()];
        let csv = csv::generic(out).await?;
        Ok(csv)
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
