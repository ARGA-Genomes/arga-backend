use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::http::Error;
use crate::http::Context as State;
use crate::index::lists;
use crate::index::lists::{Filters, GetListNames, GetListStats, ListDataSummary, ListStats, Pagination};
use crate::database::{schema, Database};
use crate::database::models::{NameList, Name as ArgaName};
use super::common::SpeciesCard;
use super::common::SpeciesPhoto;
use super::common::Taxonomy;
use super::helpers::SpeciesHelper;


#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum FilterType {
    Kingdom,
    Phylum,
}

#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum FilterAction {
    Include,
    Exclude,
}

#[derive(Debug, InputObject)]
pub struct FilterItem {
    filter: FilterType,
    action: FilterAction,
    value: String,
}

impl From<FilterItem> for lists::FilterItem {
    fn from(item: FilterItem) -> Self {
        let filter = match item.filter {
            FilterType::Kingdom => lists::Filter::Kingdom(item.value),
            FilterType::Phylum => lists::Filter::Phylum(item.value),
        };

        match item.action {
            FilterAction::Include => lists::FilterItem::Include(filter),
            FilterAction::Exclude => lists::FilterItem::Exclude(filter),
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct ListSpecies {
    pub taxonomy: Taxonomy,
    pub photo: Option<SpeciesPhoto>,
    pub data_summary: ListDataSummary,
}


pub struct Lists {
    pub list: NameList,
    pub names: Vec<ArgaName>,
    pub filters: Filters,
}

#[Object]
impl Lists {
    #[graphql(skip)]
    pub async fn new(
        db: &Database,
        name: String,
        filters: Filters,
        pagination: Pagination
    ) -> Result<Lists, Error>
    {
        use schema::name_lists as lists;
        let mut conn = db.pool.get().await?;

        let list = lists::table
            .filter(lists::name.eq(&name))
            .get_result::<NameList>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = list {
            return Err(Error::NotFound(name));
        }

        let list = list?;
        let names = db.list_names(&list, &filters, &pagination).await?;

        Ok(Lists { list, names, filters })
    }

    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>) -> Result<Vec<SpeciesCard>, Error> {
        let state = ctx.data::<State>().unwrap();
        let helper = SpeciesHelper::new(&state.database);

        let taxa = state.database.lists.list_taxa(&self.names).await?;
        let cards = helper.cards(taxa).await?;
        Ok(cards)
    }

    #[instrument(skip(self, ctx))]
    async fn stats(&self, ctx: &Context<'_>) -> Result<ListStats, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.list_stats(&self.list, &self.filters).await?;
        Ok(stats)
    }
}
