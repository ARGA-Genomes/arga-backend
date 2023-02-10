use async_graphql::*;
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::http::Context as State;
use crate::index::filters::{TaxonomyFilters, Filterable};
use crate::index::search::SearchFilterItem;
use crate::index::search::{Searchable, SearchResults};


pub struct Search;

#[Object]
impl Search {
    async fn filters(&self, ctx: &Context<'_>) -> Result<FilterTypeResults, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.provider.taxonomy_filters().await.unwrap();

        Ok(FilterTypeResults {
            taxonomy,
        })
    }

    async fn filtered(
        &self,
        ctx: &Context<'_>,
        kingdom: Option<String>,
        phylum: Option<String>,
        class: Option<String>,
        family: Option<String>,
        genus: Option<String>,
    ) -> Result<SearchResults, Error> {
        let state = ctx.data::<State>().unwrap();

        let mut filters = Vec::new();
        if let Some(value) = kingdom { filters.push(SearchFilterItem { field: "kingdom".into(), value })}
        if let Some(value) = phylum { filters.push(SearchFilterItem { field: "phylum".into(), value })}
        if let Some(value) = class { filters.push(SearchFilterItem { field: "class".into(), value })}
        if let Some(value) = family { filters.push(SearchFilterItem { field: "family".into(), value })}
        if let Some(value) = genus { filters.push(SearchFilterItem { field: "genus".into(), value })}

        let results = state.provider.filtered(&filters).await.unwrap();
        Ok(results)
    }
}


#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct FilterTypeResults {
    /// Filters to narrow down specimens by taxonomic rank
    pub taxonomy: TaxonomyFilters,
}
