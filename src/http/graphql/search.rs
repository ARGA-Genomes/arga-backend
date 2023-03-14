use async_graphql::*;
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::http::Context as State;
use crate::index::filters::{TaxonomyFilters, Filterable};
use crate::index::search::SearchFilterItem;
use crate::index::search::SearchFilterMethod;
use crate::index::search::SearchItem;
use crate::index::search::SearchSuggestion;
use crate::index::search::SpeciesSearch;
use crate::index::search::SpeciesSearchByCanonicalName;
use crate::index::search::SpeciesSearchExcludingCanonicalName;
use crate::index::search::{Searchable, TaxaSearch, SearchResults};


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
            ala_filters.push(SearchFilterItem { field: "kingdom_s".into(), value, method: SearchFilterMethod::Include  });
        }
        if let Some(value) = phylum {
            ala_filters.push(SearchFilterItem { field: "phylum_s".into(), value, method: SearchFilterMethod::Include  });
        }
        if let Some(value) = class {
            ala_filters.push(SearchFilterItem { field: "class_s".into(), value, method: SearchFilterMethod::Include  });
        }
        if let Some(value) = family {
            ala_filters.push(SearchFilterItem { field: "family_s".into(), value, method: SearchFilterMethod::Include  });
        }
        if let Some(value) = genus {
            ala_filters.push(SearchFilterItem { field: "rk_genus".into(), value, method: SearchFilterMethod::Include  });
        }

        // get a list of species from the ALA species endpoint first. once we have that
        // look for exact matches by id in the ARGA index to determine if it has any genomic data
        let mut ala_results = state.ala_provider.filtered(&ala_filters).await.unwrap();

        // create a solr filter that specifically looks for the ids found in the ALA index
        let mut solr_filters = Vec::with_capacity(ala_results.records.len());
        for record in &ala_results.records {
            if let Some(uuid) = &record.species_uuid {
                let uuid = format!(r#"("{}")"#, uuid);
                solr_filters.push(SearchFilterItem { field: "speciesID".into(), value: uuid, method: SearchFilterMethod::Include  });
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
            db_filters.push(SearchFilterItem { field: "kingdom".into(), value, method: SearchFilterMethod::Include });
        }
        if let Some(value) = phylum {
            db_filters.push(SearchFilterItem { field: "phylum".into(), value, method: SearchFilterMethod::Include  });
        }
        if let Some(value) = class {
            db_filters.push(SearchFilterItem { field: "class".into(), value, method: SearchFilterMethod::Include  });
        }
        if let Some(value) = family {
            db_filters.push(SearchFilterItem { field: "family".into(), value, method: SearchFilterMethod::Include  });
        }
        if let Some(value) = genus {
            db_filters.push(SearchFilterItem { field: "genus".into(), value, method: SearchFilterMethod::Include  });
        }

        // limit the results to 20 for pagination. this should become variable
        // once a pagination system is more fleshed out
        let mut results = Vec::with_capacity(21);

        // first get the data we do have from the solr index.
        let solr_results = state.provider.search_species("", &db_filters).await.unwrap();

        for record in solr_results.records.into_iter().take(21) {
            results.push(SearchItem {
                id: record.species_name.clone(),
                genomic_data_records: Some(record.total_records),
                scientific_name: Some(record.species_name.clone()),
                canonical_name: Some(record.species_name),
                ..SearchItem::default()
            });
        }

        // get species from gbif backbone that don't have any genomic records
        let db_results = state.db_provider.filtered(&db_filters).await.unwrap();

        for mut record in db_results.records.into_iter() {
            if let None = results.iter().find(|r| r.canonical_name == record.canonical_name) {
                record.genomic_data_records = Some(0);
                results.push(record);
            }
        }

        // sort by the amount of the genomic records. the database results should be
        // sorted by scientific name already so the secondary order should be by name
        results.sort_by(|a, b| {
            b.genomic_data_records.cmp(&a.genomic_data_records)
        });

        Ok(SearchResults {
            total: db_results.total,
            records: results.into_iter().take(21).collect(),
        })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn suggestions(&self, ctx: &Context<'_>, query: String) -> Result<Vec<SearchSuggestion>, Error> {
        tracing::info!(monotonic_counter.suggestions_made = 1);

        let state = ctx.data::<State>().unwrap();
        // let suggestions = state.ala_provider.suggestions(&query).await.unwrap();
        let suggestions = state.db_provider.suggestions(&query).await.unwrap();

        tracing::info!(value.suggestions = suggestions.len());

        Ok(suggestions)
    }
}


#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct FilterTypeResults {
    /// Filters to narrow down specimens by taxonomic rank
    pub taxonomy: TaxonomyFilters,
}
