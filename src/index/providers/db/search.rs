use async_trait::async_trait;
use uuid::Uuid;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;

use crate::index::search::{
    Searchable,
    SearchResults,
    SearchFilterItem,
    SearchItem,
    SpeciesList,
    SearchSuggestion,
    TaxaSearch,
    SpeciesSearch,
    SpeciesSearchResult,
    SpeciesSearchByCanonicalName,
    SearchFilterMethod,
    SpeciesSearchExcludingCanonicalName,
    GenusSearchResult,
    GenusSearch,
    GenusSearchItem
};
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
            canonical_name: source.canonical_name.clone(),
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
            .filter(taxon_rank.eq("species"))
            .filter(taxonomic_status.eq("accepted"))
            .order_by(canonical_name)
            .limit(21)
            .into_boxed();

        // we mutate the query variable through the loop to build the proper
        // filter but this is only possible because the query was boxed. this
        // means the query cant be inlined by the compiler but the performance
        // impact should be negligible
        for filter in filters.into_iter() {
            query = match filter.method {
                SearchFilterMethod::Include => match filter.field.as_str() {
                    "kingdom" => query.filter(kingdom.eq(&filter.value)),
                    "phylum" => query.filter(phylum.eq(&filter.value)),
                    "class" => query.filter(class.eq(&filter.value)),
                    "order" => query.filter(order.eq(&filter.value)),
                    "family" => query.filter(family.eq(&filter.value)),
                    "genus" => query.filter(genus.eq(&filter.value)),
                    _ => query,
                },
                SearchFilterMethod::Exclude => match filter.field.as_str() {
                    "kingdom" => query.filter(kingdom.ne(&filter.value)),
                    "phylum" => query.filter(phylum.ne(&filter.value)),
                    "class" => query.filter(class.ne(&filter.value)),
                    "order" => query.filter(order.ne(&filter.value)),
                    "family" => query.filter(family.ne(&filter.value)),
                    "genus" => query.filter(genus.ne(&filter.value)),
                    _ => query,
                },
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
        use crate::schema_gnl::gnl::dsl::*;

        if query.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.pool.get().await?;

        let rows = gnl
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


#[async_trait]
impl SpeciesSearchByCanonicalName for Database {
    type Error = Error;

    async fn search_species_by_canonical_names(&self, names: Vec<String>) -> Result<SpeciesSearchResult, Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let rows = taxa
            .select(scientific_name)
            .filter(taxon_rank.eq("species"))
            .filter(taxonomic_status.eq("accepted"))
            .filter(canonical_name.eq_any(names))
            .order_by(canonical_name)
            .load::<Option<String>>(&mut conn).await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            if let Some(name) = row {
                items.push(crate::index::search::SpeciesSearchItem {
                    scientific_name: None,
                    canonical_name: Some(name),
                    total_records: 0,
                });
            }
        }

        Ok(SpeciesSearchResult {
            records: items,
        })
    }
}


#[async_trait]
impl SpeciesSearchExcludingCanonicalName for Database {
    type Error = Error;

    async fn search_species_excluding_canonical_names(&self, names: Vec<String>) -> Result<SpeciesSearchResult, Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let rows = taxa
            .select(scientific_name)
            .filter(taxon_rank.eq("species"))
            .filter(taxonomic_status.eq("accepted"))
            .filter(canonical_name.ne_all(names))
            .order_by(canonical_name)
            .load::<Option<String>>(&mut conn).await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            if let Some(name) = row {
                items.push(crate::index::search::SpeciesSearchItem {
                    scientific_name: None,
                    canonical_name: Some(name),
                    total_records: 0,
                });
            }
        }

        Ok(SpeciesSearchResult {
            records: items,
        })
    }
}


#[async_trait]
impl GenusSearch for Database {
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn search_genus(&self, _query: &str, filters: &Vec<SearchFilterItem>) -> Result<GenusSearchResult, Self::Error> {
        use diesel::dsl::count_star;
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let mut query = taxa
            .group_by(genus)
            .select((genus, count_star()))
            .filter(taxonomic_status.eq("accepted"))
            .filter(taxon_rank.eq("genus"))
            .order_by(genus)
            .limit(21)
            .into_boxed();

        // we mutate the query variable through the loop to build the proper
        // filter but this is only possible because the query was boxed. this
        // means the query cant be inlined by the compiler but the performance
        // impact should be negligible
        for filter in filters.into_iter() {
            query = match filter.method {
                SearchFilterMethod::Include => match filter.field.as_str() {
                    "kingdom" => query.filter(kingdom.eq(&filter.value)),
                    "phylum" => query.filter(phylum.eq(&filter.value)),
                    "class" => query.filter(class.eq(&filter.value)),
                    "order" => query.filter(order.eq(&filter.value)),
                    "family" => query.filter(family.eq(&filter.value)),
                    "genus" => query.filter(genus.eq(&filter.value)),
                    _ => query,
                },
                SearchFilterMethod::Exclude => match filter.field.as_str() {
                    "kingdom" => query.filter(kingdom.ne(&filter.value)),
                    "phylum" => query.filter(phylum.ne(&filter.value)),
                    "class" => query.filter(class.ne(&filter.value)),
                    "order" => query.filter(order.ne(&filter.value)),
                    "family" => query.filter(family.ne(&filter.value)),
                    "genus" => query.filter(genus.ne(&filter.value)),
                    _ => query,
                },
            };
        }

        tracing::debug!(query = %diesel::debug_query(&query));
        let rows = query.load::<(Option<String>, i64)>(&mut conn).await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            if let (Some(name), total) = row {
                items.push(GenusSearchItem {
                    genus_name: name,
                    total_records: total as usize,
                });
            }
        }

        Ok(GenusSearchResult {
            records: items,
        })
    }
}


#[async_trait]
impl SpeciesSearch for Database {
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn search_species(&self, _query: &str, filters: &Vec<SearchFilterItem>) -> Result<SpeciesSearchResult, Self::Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let mut query = taxa
            .select((scientific_name, canonical_name))
            .filter(taxonomic_status.eq("accepted"))
            .filter(taxon_rank.eq("species"))
            .order_by(scientific_name)
            .limit(21)
            .into_boxed();

        // we mutate the query variable through the loop to build the proper
        // filter but this is only possible because the query was boxed. this
        // means the query cant be inlined by the compiler but the performance
        // impact should be negligible
        for filter in filters.into_iter() {
            query = match filter.method {
                SearchFilterMethod::Include => match filter.field.as_str() {
                    "kingdom" => query.filter(kingdom.eq(&filter.value)),
                    "phylum" => query.filter(phylum.eq(&filter.value)),
                    "class" => query.filter(class.eq(&filter.value)),
                    "order" => query.filter(order.eq(&filter.value)),
                    "family" => query.filter(family.eq(&filter.value)),
                    "genus" => query.filter(genus.eq(&filter.value)),
                    _ => query,
                },
                SearchFilterMethod::Exclude => match filter.field.as_str() {
                    "kingdom" => query.filter(kingdom.ne(&filter.value)),
                    "phylum" => query.filter(phylum.ne(&filter.value)),
                    "class" => query.filter(class.ne(&filter.value)),
                    "order" => query.filter(order.ne(&filter.value)),
                    "family" => query.filter(family.ne(&filter.value)),
                    "genus" => query.filter(genus.ne(&filter.value)),
                    _ => query,
                },
            };
        }

        tracing::debug!(query = %diesel::debug_query(&query));
        let rows = query.load::<SpeciesSearchItem>(&mut conn).await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(row.into());
        }

        Ok(SpeciesSearchResult {
            records: items,
        })
    }
}

#[derive(Queryable, Debug)]
struct SpeciesSearchItem {
    scientific_name: Option<String>,
    canonical_name: Option<String>,
}

impl From<SpeciesSearchItem> for crate::index::search::SpeciesSearchItem {
    fn from(source: SpeciesSearchItem) -> Self {
        Self {
            scientific_name: source.scientific_name,
            canonical_name: source.canonical_name,
            total_records: 0,
        }
    }
}
