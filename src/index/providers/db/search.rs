use async_trait::async_trait;
use uuid::Uuid;

use sqlx::{Postgres, QueryBuilder, Row};

use crate::index::search::{Searchable, SearchResults, SearchFilterItem, SearchItem, SpeciesList, SearchSuggestion, TaxaSearch};
use super::{Database, Error};


#[async_trait]
impl Searchable for Database {
    type Error = Error;

    async fn filtered(&self, filters: &Vec<SearchFilterItem>) -> Result<SearchResults, Error> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(r#"
SELECT id,
       taxon_id,
       scientific_name,
       scientific_name_authorship,
       canonical_name,
       generic_name,
       kingdom,
       phylum,
       class,
       "order",
       genus
FROM taxa
WHERE taxonomic_status='accepted'
        "#);

        for filter in filters {
            match filter.field.as_str() {
                "kingdom" => {
                    builder.push("AND kingdom = ");
                    builder.push_bind(filter.value.clone());
                },
                "phylum" => {
                    builder.push("AND phylum = ");
                    builder.push_bind(filter.value.clone());
                },
                "class" => {
                    builder.push("AND class = ");
                    builder.push_bind(filter.value.clone());
                },
                "order" => {
                    builder.push("AND order = ");
                    builder.push_bind(filter.value.clone());
                },
                "genus" => {
                    builder.push("AND genus = ");
                    builder.push_bind(filter.value.clone());
                },
                _ => {}
            };
        }

        builder.push(" LIMIT 20");
        let rows = builder.build().fetch_all(&self.pool).await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(SearchItem::from(Taxon {
                id: row.try_get("id")?,
                taxon_id: row.try_get("taxon_id")?,
                scientific_name: row.try_get("scientific_name")?,
                scientific_name_authorship: row.try_get("scientific_name_authorship")?,
                canonical_name: row.try_get("canonical_name")?,
                generic_name: row.try_get("generic_name")?,
                kingdom: row.try_get("kingdom")?,
                phylum: row.try_get("phylum")?,
                class: row.try_get("class")?,
                order: row.try_get("order")?,
                genus: row.try_get("genus")?,
            }));
        }

        Ok(SearchResults {
            total: items.len(),
            records: items,
        })
    }

    async fn species(&self, filters: &Vec<SearchFilterItem>) -> Result<SpeciesList, Error> {
        Ok(SpeciesList {
            total: 0,
            groups: vec![],
        })
    }
}


#[derive(Debug)]
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


#[derive(Debug)]
struct Suggestion {
    id: Uuid,
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    matched: Option<String>,
}

impl From<Suggestion> for SearchSuggestion {
    fn from(source: Suggestion) -> Self {
        Self {
            guid: source.id.to_string(),
            species_name: source.scientific_name.unwrap_or_default(),
            common_name: source.canonical_name,
            matched: source.matched.unwrap_or_default(),
        }
    }
}


#[async_trait]
impl TaxaSearch for Database {
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn suggestions(&self, query: &str) ->  Result<Vec<SearchSuggestion> ,Self::Error> {
        if query.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query_as!(Suggestion, r#"
SELECT id,
       scientific_name,
       canonical_name,
       scientific_name AS matched
FROM taxa
WHERE canonical_name ILIKE $1
AND taxon_rank='species'
ORDER BY scientific_name ASC
LIMIT 5
        "#, format!("%{query}%")).fetch_all(&self.pool).await?;

        tracing::info!(?rows);
        let suggestions = rows.into_iter().map(|r| SearchSuggestion::from(r)).collect();
        Ok(suggestions)
    }
}
