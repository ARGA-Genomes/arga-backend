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


        state.db_provider.species(&ala_filters).await.unwrap();

        Ok(ala_results)
    }

    async fn filtered2(
        &self,
        ctx: &Context<'_>,
        kingdom: Option<String>,
        phylum: Option<String>,
        class: Option<String>,
        family: Option<String>,
        genus: Option<String>,
    ) -> Result<SearchResults, Error> {
        let state = ctx.data::<State>().unwrap();

        let mut db_filters = Vec::new();

        if let Some(value) = kingdom {
            db_filters.push(SearchFilterItem { field: "kingdom".into(), value });
        }
        if let Some(value) = phylum {
            db_filters.push(SearchFilterItem { field: "phylum".into(), value });
        }
        if let Some(value) = class {
            db_filters.push(SearchFilterItem { field: "class".into(), value });
        }
        if let Some(value) = family {
            db_filters.push(SearchFilterItem { field: "family".into(), value });
        }
        if let Some(value) = genus {
            db_filters.push(SearchFilterItem { field: "genus".into(), value });
        }

        // get a list of species from the database first. once we have that we can
        // look for genomic data related to the species and enrich the search results
        let mut db_results = state.db_provider.filtered(&db_filters).await.unwrap();

        // use the same search filters in solr since the field names are the same
        let solr_results = state.provider.species(&db_filters).await.unwrap();

        for record in db_results.records.iter_mut() {
            let mut total_genomic_records = 0;

            for group in solr_results.groups.iter() {
                if group.key == record.species {
                    total_genomic_records += group.matches;
                }
            }

            record.genomic_data_records = Some(total_genomic_records);
        }

        Ok(db_results)
    }
}


#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct FilterTypeResults {
    /// Filters to narrow down specimens by taxonomic rank
    pub taxonomy: TaxonomyFilters,
}
