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
    GenusSearchItem, SpeciesSearchWithRegion
};
use super::{schema, schema_gnl, Database, Error};
use super::models::ArgaTaxon;


#[derive(Queryable, Debug)]
struct Taxon {
    id: Uuid,
    _taxon_id: Option<i64>,

    scientific_name: Option<String>,
    _scientific_name_authorship: Option<String>,
    canonical_name: Option<String>,
    _generic_name: Option<String>,

    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    _family: Option<String>,
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
        use schema::taxa::dsl::*;
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
    scientific_name: String,
    canonical_name: Option<String>,
}

impl From<Suggestion> for SearchSuggestion {
    fn from(source: Suggestion) -> Self {
        Self {
            guid: source.id.to_string(),
            species_name: source.scientific_name.clone(),
            common_name: source.canonical_name,
            matched: source.scientific_name.clone(),
        }
    }
}


#[async_trait]
impl TaxaSearch for Database {
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn suggestions(&self, query: &str) ->  Result<Vec<SearchSuggestion> ,Self::Error> {
        use schema::names::dsl::*;

        if query.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.pool.get().await?;
        let rows = names
            .select((
                id,
                scientific_name,
                canonical_name,
            ))
            .limit(5)
            .order_by(scientific_name.distance(query))
            .load::<Suggestion>(&mut conn)
            .await?;

        let suggestions = rows.into_iter().map(|r| r.into()).collect();
        Ok(suggestions)
    }
}


#[async_trait]
impl SpeciesSearchByCanonicalName for Database {
    type Error = Error;

    async fn search_species_by_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Error> {
        use schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        let rows = gnl
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
                    total_genomic_records: None,
                    data_summary: Default::default(),
                    photo: Default::default()
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

    async fn search_species_excluding_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Error> {
        use schema::taxa::dsl::*;
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
                    total_genomic_records: None,
                    data_summary: Default::default(),
                    photo: Default::default()
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
        use schema::taxa::dsl::*;
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
    async fn search_species(&self, q: Option<String>, filters: &Vec<SearchFilterItem>, results_type: Option<WithRecordType>) -> Result<SpeciesSearchResult, Self::Error> {
        use schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        let mut query = gnl
            .select((scientific_name, canonical_name))
            .filter(taxonomic_status.eq("accepted"))
            .filter(taxon_rank.eq("species"))
            .order_by(scientific_name)
            .limit(21)
            .into_boxed();

        if let Some(q) = q {
            let q = format!("%{}%", q);
            query = query.filter(scientific_name.ilike(q))
        }

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

#[async_trait]
impl SpeciesSearchWithRegion for Database {
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn search_species_with_region(
        &self,
        region: &Vec<String>,
        filters: &Vec<SearchFilterItem>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<ArgaTaxon>, Self::Error>
    {
        use schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        use schema_gnl::eav_arrays::dsl::*;

        let mut query = gnl
            .inner_join(eav_arrays.on(entity_id.eq(id)))
            .select(schema_gnl::gnl::all_columns)
            .filter(name.eq_any(vec!["ibraRegions", "imcraRegions"]))
            .filter(value.overlaps_with(region))
            .order_by(canonical_name)
            .offset(offset)
            .limit(limit)
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
        let rows = query.load::<ArgaTaxon>(&mut conn).await?;

        Ok(rows)
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
            total_genomic_records: None,
            data_summary: Default::default(),
            photo: Default::default()
        }
    }
}



// Diesel extensions for postgres trigrams
use diesel::sql_types::{Float, Varchar, Nullable};
use diesel::expression::{AsExpression, Expression};

sql_function! {
    fn word_similarity(x: Varchar, y: Varchar) -> Float;
}

use diesel::pg::Pg;
use diesel::sql_types::*;
use crate::http::graphql::search::WithRecordType;

diesel::infix_operator!(Similar, " % ", Bool, backend: Pg);
diesel::infix_operator!(WordSimilar, " <% ", Bool, backend: Pg);
diesel::infix_operator!(WordSimilarComm, " %> ", Bool, backend: Pg);
diesel::infix_operator!(Distance, " <-> ", Float, backend: Pg);
diesel::infix_operator!(WordDistance, " <<-> ", Float, backend: Pg);
diesel::infix_operator!(WordDistanceComm, " <->> ", Float, backend: Pg);


pub trait TrgmQueryExtensions: Expression + Sized {
  fn distance<T: AsExpression<Text>>(self, other: T) -> WordDistanceComm<Self, T::Expression> {
    WordDistanceComm::new(self, other.as_expression())
  }
}


pub trait TrgmQueryExtensionsInner {}

impl TrgmQueryExtensionsInner for Text {}
impl TrgmQueryExtensionsInner for Nullable<Text> {}

impl<U: TrgmQueryExtensionsInner, T: Expression<SqlType = U>> TrgmQueryExtensions for T {}
