use async_trait::async_trait;
use uuid::Uuid;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;

use crate::index::search::{Searchable, SearchResults, SearchFilterItem, SearchItem, SpeciesList, SearchSuggestion, TaxaSearch};
use super::{Database, Error};


#[derive(Queryable, Debug)]
struct Taxon {
    id: Uuid,
    taxon_id: Option<i64>,

    scientific_name: Option<String>,
    scientific_name_authorship: Option<String>,
    canonical_name: Option<String>,
    generic_name: Option<String>,

    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    genus: Option<String>,
}

impl From<Taxon> for SearchItem {
    fn from(source: Taxon) -> Self {
        SearchItem {
            id: source.id.to_string(),
            species_uuid: None,
            genomic_data_records: None,
            scientific_name: source.scientific_name,
            genus: source.genus,
            subgenus: None,
            kingdom: source.kingdom,
            phylum: source.phylum,
            family: source.order,
            class: source.class,
            species: source.canonical_name,
            species_group: None,
            species_subgroup: None,
            biome: None,
            event_date: None,
            event_time: None,
            license: None,
            recorded_by: None,
            identified_by: None,
        }
    }
}


#[async_trait]
impl Searchable for Database {
    type Error = Error;

    async fn filtered(&self, filters: &Vec<SearchFilterItem>) -> Result<SearchResults, Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let mut query = taxa
            .select((
                id,
                taxon_id,
                scientific_name,
                scientific_name_authorship,
                canonical_name,
                generic_name,
                kingdom,
                phylum,
                class,
                order,
                family,
                genus,
            ))
            .filter(taxonomic_status.eq("accepted"))
            .limit(20)
            .into_boxed();

        // we mutate the query variable through the loop to build the proper
        // filter but this is only possible because the query was boxed. this
        // means the query cant be inlined by the compiler but the performance
        // impact should be negligible
        for filter in filters.into_iter() {
            query = match filter.field.as_str() {
                "kingdom" => query.filter(kingdom.eq(&filter.value)),
                "phylum" => query.filter(phylum.eq(&filter.value)),
                "class" => query.filter(class.eq(&filter.value)),
                "order" => query.filter(order.eq(&filter.value)),
                "family" => query.filter(family.eq(&filter.value)),
                "genus" => query.filter(genus.eq(&filter.value)),
                _ => query,
            };
        }

        let rows = query.load::<Taxon>(&mut conn).await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(SearchItem::from(row));
        }

        Ok(SearchResults {
            total: items.len(),
            records: items,
        })
    }

    async fn species(&self, _filters: &Vec<SearchFilterItem>) -> Result<SpeciesList, Error> {
        Ok(SpeciesList {
            total: 0,
            groups: vec![],
        })
    }
}


#[derive(Queryable, Debug)]
struct Suggestion {
    id: Uuid,
    scientific_name: Option<String>,
    canonical_name: Option<String>,
}

impl From<Suggestion> for SearchSuggestion {
    fn from(source: Suggestion) -> Self {
        Self {
            guid: source.id.to_string(),
            species_name: source.scientific_name.clone().unwrap_or_default(),
            common_name: source.canonical_name,
            matched: source.scientific_name.unwrap_or_default(),
        }
    }
}


#[async_trait]
impl TaxaSearch for Database {
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn suggestions(&self, query: &str) ->  Result<Vec<SearchSuggestion> ,Self::Error> {
        use crate::schema::taxa::dsl::*;

        if query.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.pool.get().await?;

        let rows = taxa
            .select((
                id,
                scientific_name,
                canonical_name,
            ))
            .filter(taxon_rank.eq("species"))
            .filter(canonical_name.ilike(format!("%{query}%")))
            .order(scientific_name.asc())
            .limit(5)
            .load::<Suggestion>(&mut conn).await?;

        let suggestions = rows.into_iter().map(|r| r.into()).collect();
        Ok(suggestions)
    }
}
