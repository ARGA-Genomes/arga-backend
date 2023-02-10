use async_trait::async_trait;
use serde::Deserialize;

use crate::index::filters::{Filterable, TaxonomyFilters, Filter, FilterOption};
use super::{Solr, Error};


#[async_trait]
impl Filterable for Solr {
    type Error = Error;

    async fn taxonomy_filters(&self) -> Result<TaxonomyFilters, Error> {
        let params = vec![
            ("q", "*:*"),
            ("fl", "id"),
            ("rows", "300"),
            ("group", "true"),
            ("group.field", "kingdom"),
            ("group.field", "phylum"),
            ("group.field", "family"),
            ("group.field", "class"),
            ("group.field", "genus"),
        ];

        let results = self.client.select::<Fields>(&params).await?;

        Ok(TaxonomyFilters {
            kingdom: Some(results.kingdom.into()),
            phylum: Some(results.phylum.into()),
            class: Some(results.class.into()),
            family: Some(results.family.into()),
            genus: Some(results.genus.into()),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fields {
    kingdom: Matches,
    phylum: Matches,
    class: Matches,
    family: Matches,
    genus: Matches,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Matches {
    /// The amount of matched records
    matches: usize,
    /// The amount of records ascribed to the category
    groups: Vec<Group>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Group {
    group_value: Option<String>,
    doclist: DocList,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocList {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
}

impl From<Matches> for Filter {
    fn from(source: Matches) -> Self {
        let mut filters = Vec::with_capacity(source.groups.len());
        for group in source.groups {
            filters.push(FilterOption {
                matches: group.doclist.total,
                value: group.group_value,
            })
        }

        Filter {
            total_matches: source.matches,
            values: filters,
        }
    }
}
