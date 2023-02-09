use axum::async_trait;
use thiserror::Error;
use serde::{de::DeserializeOwned, Deserialize};

use crate::index::filters::{self, Filterable, TaxonomyFilters, FilterOption};


#[derive(Error, Debug)]
pub enum Error {
    #[error("internal request error")]
    Request(#[from] reqwest::Error),
}


#[derive(Clone)]
pub struct Solr {
    client: SolrClient,
}


impl Solr {
    pub fn new(client: SolrClient) -> Solr {
        Solr { client }
    }
}


#[async_trait]
impl Filterable for Solr {
    type Error = Error;

    async fn taxonomy_filters(&self) -> Result<TaxonomyFilters, Error> {
        let query = format!(r#"*%3A*&fl=id&group=true&group.field=phylum&group.field=kingdom&group.field=family&group.field=class&group.field=genus"#);

        let results = self.client.select_grouped::<Fields>(&query, 200).await?;

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

impl From<Matches> for filters::Filter {
    fn from(source: Matches) -> Self {
        let mut filters = Vec::with_capacity(source.groups.len());
        for group in source.groups {
            filters.push(FilterOption {
                matches: group.doclist.total,
                value: group.group_value,
            })
        }

        filters::Filter {
            total_matches: source.matches,
            values: filters,
        }
    }
}


#[derive(Deserialize)]
struct SolrResult<T> {
    response: T,
}

#[derive(Deserialize)]
struct SolrGroupResult<T> {
    grouped: T,
}

#[derive(Clone)]
pub struct SolrClient {
    host: String,
    client: reqwest::Client,
}

impl SolrClient {
    pub fn new(host: &str) -> SolrClient {
        SolrClient {
            host: String::from(host),
            client: reqwest::Client::new(),
        }
    }

    pub async fn select<'a, T>(&self, query: &str, rows: usize) -> Result<T, Error>
        where T: DeserializeOwned + std::fmt::Debug
    {
        let url = format!("{}/select?q={query}&rows={rows}", self.host);
        let resp = self.client.get(url).send().await?;
        let json = resp.json::<SolrResult<T>>().await;
        Ok(json?.response)
    }

    pub async fn select_grouped<'a, T>(&self, query: &str, rows: usize) -> Result<T, Error>
        where T: DeserializeOwned + std::fmt::Debug
    {
        let url = format!("{}/select?q={query}&rows={rows}", self.host);
        let resp = self.client.get(url.clone()).send().await?;
        let json = resp.json::<SolrGroupResult<T>>().await;
        Ok(json?.grouped)
    }
}
