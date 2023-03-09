use async_trait::async_trait;
use serde::Deserialize;

use crate::index::stats::{GetGenusStats, GenusStats, GenusBreakdown, GetGenusBreakdown, GenusBreakdownItem};
use super::{Solr, Error};


#[async_trait]
impl GetGenusStats for Solr {
    type Error = Error;

    async fn genus_stats(&self, genus: &str) -> Result<GenusStats, Error> {
        let breakdown = self.species_breakdown(genus).await?;
        Ok(GenusStats {
            // TODO: change the result struct to be usize so there isn't a potential overflow panic
            total_species: breakdown.species.len() as i64,
        })
    }
}

#[async_trait]
impl GetGenusBreakdown for Solr {
    type Error = Error;

    async fn species_breakdown(&self, genus: &str) -> Result<GenusBreakdown, Error> {
        let filter = &format!("genus:{genus}");

        // get all species that have a matched name. this filters
        // out records that couldn't be matched by the name-matching service
        let params = vec![
            ("q", "*:*"),
            ("rows", "0"),
            ("fq", filter),
            ("fq", r#"taxonRank:"species""#),
            ("facet", "true"),
            ("facet.pivot", "scientificName"),
        ];

        tracing::debug!(?params);
        let (_, mut facets) = self.client.select_faceted::<DataRecords, SpeciesFacet>(&params).await?;

        // species that couldn't be name matched will appear in the index
        // with a taxonRank of genus and a higherMatch type. this lets us
        // combine unmatched species whilst still retaining the normalised species
        // from the name matching service
        let params = vec![
            ("q", "*:*"),
            ("rows", "0"),
            ("fq", filter),
            ("fq", r#"taxonRank:"genus""#),
            ("fq", r#"matchType:"higherMatch""#),
            ("facet", "true"),
            ("facet.pivot", "raw_scientificName"),
        ];

        tracing::debug!(?params);
        let (_, raw_facets) = self.client.select_faceted::<DataRecords, RawSpeciesFacet>(&params).await?;

        facets.scientific_name.extend(raw_facets.scientific_name);

        Ok(GenusBreakdown {
            species: facets.scientific_name.into_iter().map(|s| s.into()).collect(),
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
#[serde(rename_all = "camelCase")]
struct SpeciesFacet {
    scientific_name: Vec<Facet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSpeciesFacet {
    #[serde(rename(deserialize = "raw_scientificName"))]
    scientific_name: Vec<Facet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
