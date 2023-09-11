use async_trait::async_trait;
use uuid::Uuid;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;

use crate::database::models::TaxonomicStatus;
use crate::index::search::{
    SearchItem,
    SpeciesSearchResult,
    SpeciesSearchByCanonicalName,
    SpeciesSearchExcludingCanonicalName,
};
use super::{schema_gnl, Database, Error};


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
impl SpeciesSearchByCanonicalName for Database {
    type Error = Error;

    async fn search_species_by_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let rows = species
            .select(scientific_name)
            .filter(status.eq(TaxonomicStatus::Accepted))
            .filter(canonical_name.eq_any(names))
            .order_by(canonical_name)
            .load::<String>(&mut conn).await?;

        let mut items = Vec::with_capacity(rows.len());
        for name in rows {
            items.push(crate::index::search::SpeciesSearchItem {
                scientific_name: None,
                canonical_name: Some(name),
                total_records: 0,
                total_genomic_records: None,
                data_summary: Default::default()
            });
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
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let rows = species
            .select(scientific_name)
            .filter(status.eq(TaxonomicStatus::Accepted))
            .filter(canonical_name.ne_all(names))
            .order_by(canonical_name)
            .load::<String>(&mut conn).await?;

        let mut items = Vec::with_capacity(rows.len());
        for name in rows {
            items.push(crate::index::search::SpeciesSearchItem {
                scientific_name: None,
                canonical_name: Some(name),
                total_records: 0,
                total_genomic_records: None,
                data_summary: Default::default(),
            });
        }

        Ok(SpeciesSearchResult {
            records: items,
        })
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
