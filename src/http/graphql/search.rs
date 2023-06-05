use std::collections::HashMap;

use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::http::Context as State;
use crate::index::filters::{TaxonomyFilters, Filterable};
use crate::index::search::{
    DNASearchByCanonicalName,
    FullTextSearch,
    FullTextSearchItem,
    FullTextSearchResult,
    FullTextType,
    GenusSearch,
    GenusSearchItem,
    SearchFilterItem,
    SearchFilterMethod,
    SearchItem,
    SearchSuggestion,
    SpeciesSearch,
    SpeciesSearchByCanonicalName,
    SpeciesSearchItem,
    SpeciesSearchWithRegion,
    Searchable,
    TaxaSearch,
    SearchResults
};
use crate::database::schema_gnl;
use crate::database::models::ArgaTaxon;


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
        let solr_results = state.provider.search_species(None, &db_filters).await.unwrap();

        for record in solr_results.records.into_iter().take(21) {
            results.push(SearchItem {
                id: record.canonical_name.clone().unwrap(),
                genomic_data_records: Some(record.total_records),
                scientific_name: record.scientific_name,
                canonical_name: record.canonical_name,
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


    #[tracing::instrument(skip(self, ctx))]
    async fn genus(
        &self,
        ctx: &Context<'_>,
        kingdom: Option<String>,
        phylum: Option<String>,
        class: Option<String>,
        family: Option<String>,
    ) -> Result<Vec<GenusSearchItem>, Error> {
        let state = ctx.data::<State>().unwrap();
        let filters = create_filters(kingdom, phylum, class, family, None, None);
        let results = state.db_provider.search_genus("", &filters).await.unwrap();

        Ok(results.records)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn species(
        &self,
        ctx: &Context<'_>,
        kingdom: Option<String>,
        phylum: Option<String>,
        class: Option<String>,
        family: Option<String>,
        genus: Option<String>,
    ) -> Result<Vec<SpeciesSearchItem>, Error> {
        let state = ctx.data::<State>().unwrap();
        let filters = create_filters(kingdom, phylum, class, family, genus, None);

        // limit the results for pagination. this should become variable
        // once a pagination system is more fleshed out
        let mut results = Vec::with_capacity(21);

        // first get the data we do have from the solr index.
        let solr_results = state.provider.search_species(None, &filters).await.unwrap();

        for record in solr_results.records.into_iter().take(21) {
            results.push(record);
        }

        // get species from gbif backbone that don't have any genomic records
        let db_results = state.db_provider.search_species(None, &filters).await.unwrap();

        for record in db_results.records.into_iter() {
            // add the filler gbif record if we don't have enough records with data
            if let None = results.iter().find(|r| r.canonical_name == record.canonical_name) {
                results.push(record);
            }
        }

        // sort by the amount of the genomic records. the database results should be
        // sorted by scientific name already so the secondary order should be by name
        results.sort_by(|a, b| {
            b.total_records.cmp(&a.total_records)
        });

        Ok(results.into_iter().take(21).collect())
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn species_taxonomy_order(
        &self,
        ctx: &Context<'_>,
        q: Option<String>,
        kingdom: Option<String>,
        phylum: Option<String>,
        class: Option<String>,
        family: Option<String>,
        genus: Option<String>,
    ) -> Result<Vec<SpeciesSearchItem>, Error>
    {
        let state = ctx.data::<State>().unwrap();
        let filters = create_filters(kingdom, phylum, class, family, genus, None);

        let mut results = state.db_provider.search_species(q, &filters).await.unwrap();

        let names = results
            .records
            .iter()
            .map(|r|
                 match &r.canonical_name {
                     Some(name) => name.clone(),
                     None => match &r.scientific_name {
                         Some(name) => name.clone(),
                         None => String::default(),
                     },
                 }
            ).collect();

        // first get the data we do have from the solr index.
        // let solr_results = state.provider.search_species("", &filters).await.unwrap();
        let solr_results = state.provider.search_species_by_canonical_names(&names).await.unwrap();

        for mut record in results.records.iter_mut() {
            // find the total amount of genomic records from the solr search
            if let Some(solr_record) = solr_results.records.iter().find(|r| r.canonical_name == record.canonical_name) {
                record.total_records = solr_record.total_records;
            }
        }

        Ok(results.records.into_iter().take(21).collect())
    }


    #[tracing::instrument(skip(self, ctx))]
    async fn species_by_region(
        &self,
        ctx: &Context<'_>,
        ibra_region: Vec<String>,
        offset: i64,
        limit: i64,
        kingdom: Option<String>,
        phylum: Option<String>,
        class: Option<String>,
        family: Option<String>,
        genus: Option<String>,
    ) -> Result<Vec<SpeciesSummaryResult>, Error>
    {
        let state = ctx.data::<State>().unwrap();
        let filters = create_filters(kingdom, phylum, class, family, genus, None);

        let results = state.db_provider.search_species_with_region(&ibra_region, &filters, offset, limit).await.unwrap();
        let mut results: Vec<SpeciesSummaryResult> = results.into_iter().map(|v| v.into()).collect();

        results.dedup_by(|a, b| a.canonical_name == b.canonical_name);
        let names = results.iter().map(|r| r.canonical_name.clone().unwrap()).collect();

        // first get the data we do have from the solr index.
        // let solr_results = state.provider.search_species("", &filters).await.unwrap();
        let solr_results = state.provider.search_species_by_canonical_names(&names).await.unwrap();

        for mut record in results.iter_mut() {
            // find the total amount of genomic records from the solr search
            if let Some(solr_record) = solr_results.records.iter().find(|r| r.canonical_name == record.canonical_name) {
                record.total_records = solr_record.total_records;
            }
        }

        let dna_results = state.provider.search_dna_by_canonical_names(&names).await.unwrap();
        for mut record in results.iter_mut() {
            // find the total amount of dna records from the solr search
            if let Some(solr_record) = dna_results.records.iter().find(|r| r.canonical_name == record.canonical_name) {
                record.total_barcodes = solr_record.total_records;
            }
        }

        Ok(results)
    }


    async fn full_text(&self, ctx: &Context<'_>, query: String, data_type: Option<String>) -> Result<FullTextSearchResult, Error>
    {
        let data_type = data_type.unwrap_or("all".to_string());

        let state = ctx.data::<State>().unwrap();
        let mut results = FullTextSearchResult::default();

        if data_type == "all" || data_type == "species" {
            let db_results = state.search.full_text(&query).await?;
            results.records.extend(db_results.records);
        };

        // get the taxon names to enrich the result data with
        let mut names = Vec::with_capacity(results.records.len());
        for record in &results.records {
            match record {
                FullTextSearchItem::Taxon(item) => names.push(item.scientific_name.clone()),
                _ => {},
            }
        }

        // match the results against the ARGA GNL
        use schema_gnl::gnl::dsl::*;
        let mut conn = state.db_provider.pool.get().await.unwrap();

        let rows = gnl
            .filter(scientific_name.eq_any(names))
            .load::<ArgaTaxon>(&mut conn).await.unwrap();

        let mut record_map: HashMap<String, ArgaTaxon> = HashMap::new();
        for row in rows {
            if let Some(name) = &row.scientific_name {
                record_map.insert(name.clone(), row);
            }
        }

        // enrich the results with the gnl data
        for result in results.records.iter_mut() {
            match result {
                FullTextSearchItem::Taxon(item) => {
                    if let Some(record) = record_map.get(&item.scientific_name) {
                        item.scientific_name_authorship = record.scientific_name_authorship.clone();
                        item.canonical_name = record.canonical_name.clone();
                        item.rank = record.taxon_rank.clone();
                        item.taxonomic_status = record.taxonomic_status.clone();
                    }
                }
                _ => {}
            };
        }

        // if we are only searching for species shortcut the request
        if data_type == "species" {
            return Ok(results);
        }


        // get the solr full text search results
        let solr_results = state.provider.full_text(&query).await?;
        results.records.extend(solr_results.records);

        // filter out the sequence types that wasn't requested
        results.records = results.records.into_iter().filter(|record| {
            data_type == "all" || data_type == match record {
                FullTextSearchItem::Taxon(_) => "species", // should already be filtered out if not 'all'
                FullTextSearchItem::GenomeSequence(item) => {
                    match item.r#type {
                        FullTextType::Taxon => "species",
                        FullTextType::ReferenceGenomeSequence => "whole_genomes",
                        FullTextType::WholeGenomeSequence => "whole_genomes",
                        FullTextType::PartialGenomeSequence => "partial_genomes",
                        FullTextType::UnknownGenomeSequence => "unknown_genomes",
                        FullTextType::Barcode => "barcodes",
                    }
                },
            }
        }).collect();

        // mix the results from multiple sources and rank them by the search score
        results.records.sort_by(|a, b| b.partial_cmp(a).unwrap());

        Ok(results)
    }
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct SpeciesSummaryResult {
    pub scientific_name: Option<String>,
    pub canonical_name: Option<String>,
    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
    pub total_records: usize,
    pub total_barcodes: usize,
}

impl From<ArgaTaxon> for SpeciesSummaryResult {
    fn from(value: ArgaTaxon) -> Self {
        SpeciesSummaryResult {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            kingdom: value.kingdom,
            phylum: value.phylum,
            class: value.class,
            order: value.order,
            family: value.family,
            genus: value.genus,
            total_records: 0,
            total_barcodes: 0,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct FilterTypeResults {
    /// Filters to narrow down specimens by taxonomic rank
    pub taxonomy: TaxonomyFilters,
}


fn create_filters(
    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    family: Option<String>,
    genus: Option<String>,
    ibra_region: Option<String>,
) -> Vec<SearchFilterItem> {
    let mut filters = Vec::new();

    if let Some(value) = kingdom {
        filters.push(SearchFilterItem { field: "kingdom".into(), value, method: SearchFilterMethod::Include });
    }
    if let Some(value) = phylum {
        filters.push(SearchFilterItem { field: "phylum".into(), value, method: SearchFilterMethod::Include });
    }
    if let Some(value) = class {
        filters.push(SearchFilterItem { field: "class".into(), value, method: SearchFilterMethod::Include });
    }
    if let Some(value) = family {
        filters.push(SearchFilterItem { field: "family".into(), value, method: SearchFilterMethod::Include });
    }
    if let Some(value) = genus {
        filters.push(SearchFilterItem { field: "genus".into(), value, method: SearchFilterMethod::Include });
    }
    if let Some(value) = ibra_region {
        filters.push(SearchFilterItem { field: "ibra_region".into(), value, method: SearchFilterMethod::Include })
    }

    filters
}
