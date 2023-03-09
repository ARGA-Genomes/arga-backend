use async_trait::async_trait;
use serde::Deserialize;

use crate::index::stats::{GetGenusStats, GenusStats, GenusBreakdown, GetGenusBreakdown, GenusBreakdownItem};
use super::{Solr, Error};


#[async_trait]
impl GetGenusStats for Solr {
    type Error = Error;

    async fn genus_stats(&self, genus: &str) -> Result<GenusStats, Error> {
        let filter = &format!("genus:{genus}");

        let params = vec![
            ("q", "*:*"),
            ("rows", "0"),
            ("fq", filter),
            ("facet", "true"),
            ("facet.pivot", "species"),
        ];

        tracing::debug!(?params);
        let (_, facets) = self.client.select_faceted::<DataRecords, SpeciesFacet>(&params).await?;

        Ok(GenusStats {
            total_species: facets.species.len() as i64,
        })
    }
}

#[async_trait]
impl GetGenusBreakdown for Solr {
    type Error = Error;

    async fn species_breakdown(&self, genus: &str) -> Result<GenusBreakdown, Error> {
        let filter = &format!("genus:{genus}");

        let params = vec![
            ("q", "*:*"),
            ("rows", "0"),
            ("fq", filter),
            ("facet", "true"),
            ("facet.pivot", "species"),
        ];

        tracing::debug!(?params);
        let (_, facets) = self.client.select_faceted::<DataRecords, SpeciesFacet>(&params).await?;

        Ok(GenusBreakdown {
            species: facets.species.into_iter().map(|s| s.into()).collect(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataRecords {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
}


#[derive(Debug, Deserialize)]
struct SpeciesFacet {
    species: Vec<Facet>,
}

#[derive(Debug, Deserialize)]
struct Facet {
    field: String,
    value: String,
    count: usize,
}

impl From<Facet> for GenusBreakdownItem {
    fn from(source: Facet) -> Self {
        GenusBreakdownItem {
            name: source.value,
            total: source.count,
        }
    }
}
