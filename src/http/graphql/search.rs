use async_graphql::*;
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::http::Context as State;
use crate::index::filters::{TaxonomyFilters, Filterable};
use crate::index::search::SearchFilterItem;
use crate::index::search::SearchSuggestion;
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

        let mut ala_filters = Vec::new();

        // create search filters to narrow down the list in the ALA species endpoint
        if let Some(value) = kingdom {
            ala_filters.push(SearchFilterItem { field: "kingdom_s".into(), value });
        }
        if let Some(value) = phylum {
            ala_filters.push(SearchFilterItem { field: "phylum_s".into(), value });
        }
        if let Some(value) = class {
            ala_filters.push(SearchFilterItem { field: "class_s".into(), value });
        }
        if let Some(value) = family {
            ala_filters.push(SearchFilterItem { field: "family_s".into(), value });
        }
        if let Some(value) = genus {
            ala_filters.push(SearchFilterItem { field: "rk_genus".into(), value });
        }

        // get a list of species from the ALA species endpoint first. once we have that
        // look for exact matches by id in the ARGA index to determine if it has any genomic data
        let mut ala_results = state.ala_provider.filtered(&ala_filters).await.unwrap();

        // create a solr filter that specifically looks for the ids found in the ALA index
        let mut solr_filters = Vec::with_capacity(ala_results.records.len());
        for record in &ala_results.records {
            if let Some(uuid) = &record.species_uuid {
                let uuid = format!(r#"("{}")"#, uuid);
                solr_filters.push(SearchFilterItem { field: "speciesID".into(), value: uuid });
            }
        }

        let results = state.provider.species(&solr_filters).await.unwrap();

        for record in ala_results.records.iter_mut() {
            let mut total_genomic_records = 0;

            for group in results.groups.iter() {
                if group.key == record.species_uuid {
                    total_genomic_records += group.matches;
                }
            }

            record.genomic_data_records = Some(total_genomic_records);
        }

        Ok(ala_results)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn suggestions(&self, ctx: &Context<'_>, query: String) -> Result<Vec<SearchSuggestion>, Error> {
        let state = ctx.data::<State>().unwrap();
        let suggestions = state.ala_provider.suggestions(&query).await.unwrap();

        tracing::info!(value.suggestions = suggestions.len());

        Ok(suggestions)
    }
}


#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct FilterTypeResults {
    /// Filters to narrow down specimens by taxonomic rank
    pub taxonomy: TaxonomyFilters,
}
