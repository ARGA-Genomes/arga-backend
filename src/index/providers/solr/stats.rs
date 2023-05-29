use std::collections::HashMap;

use async_trait::async_trait;
use serde::Deserialize;

use crate::index::{stats::{
    GetGenusStats,
    GenusStats,
    GenusBreakdown,
    GetGenusBreakdown,
    GenusBreakdownItem,
    GetFamilyStats,
    FamilyStats,
    FamilyBreakdown,
    GetFamilyBreakdown,
    FamilyBreakdownItem, GetSpeciesStats, SpeciesStats
}, providers::db::models::Name};
use super::{Solr, Error};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataRecords {
    #[serde(rename(deserialize = "numFound"))]
    _total: usize,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Facet {
    _field: String,
    value: String,
    count: usize,
    queries: HashMap<String, usize>,
}

const QUERY_WHOLE_GENOMES: &str = r#"{!tag=q1}dynamicProperties_ncbi_genome_rep:"Full""#;
const QUERY_BARCODES: &str = r#"{!tag=q1}dataProviderName:"Barcode of Life""#;


#[async_trait]
impl GetSpeciesStats for Solr {
    type Error = Error;

    async fn species_stats(&self, names: &Vec<Name>) -> Result<Vec<SpeciesStats>, Error> {
        // create a name map to link solr results up with the global names list
        let mut name_map: HashMap<String, &Name> = HashMap::new();
        for name in names {
            // because the query falls back to a scientific name if a canonical
            // name isn't available we insert both names into the map returning
            // the same name record
            name_map.insert(name.scientific_name.clone(), name);
            if let Some(canonical_name) = name.canonical_name.clone() {
                name_map.insert(canonical_name, name);
            }
        }

        // craft a single filter by joining them all with OR since the default
        // will treat it as an AND query
        let names = names.into_iter().map(|name| {
            format!("\"{}\"", name.canonical_name.clone().unwrap_or_else(|| name.scientific_name.clone()))
        }).collect::<Vec<String>>();
        let joined_names = names.join(" OR ");

        let filter = &format!("scientificName:{joined_names}");

        // get all species that have a matched name. this filters
        // out records that couldn't be matched by the name-matching service
        let params = vec![
            ("q", "*:*"),
            ("rows", "0"),
            ("fq", filter),
            ("fq", "taxonRank:species"),
            ("facet", "true"),
            ("facet.query", QUERY_WHOLE_GENOMES),
            ("facet.query", QUERY_BARCODES),
            ("facet.pivot", "{!query=q1}scientificName"),
        ];

        tracing::debug!(?params);
        let (_, facets) = self.client.select_faceted::<DataRecords, SpeciesFacet>(&params).await?;

        let mut stats = Vec::with_capacity(names.len());
        for facet in facets.scientific_name {
            if let Some(name) = name_map.get(&facet.value) {
                stats.push(SpeciesStats {
                    name: name.clone().to_owned(),
                    total: facet.count,
                    whole_genomes: *facet.queries.get(QUERY_WHOLE_GENOMES).unwrap_or(&0),
                    barcodes: *facet.queries.get(QUERY_BARCODES).unwrap_or(&0),
                    mitogenomes: 0,
                });
            }
        }

        Ok(stats)
    }
}



#[async_trait]
impl GetGenusStats for Solr {
    type Error = Error;

    async fn genus_stats(&self, genus: &str) -> Result<GenusStats, Error> {
        let breakdown = self.genus_breakdown(genus).await?;
        Ok(GenusStats {
            total_species: breakdown.species.len(),
        })
    }
}

#[async_trait]
impl GetGenusBreakdown for Solr {
    type Error = Error;

    async fn genus_breakdown(&self, genus: &str) -> Result<GenusBreakdown, Error> {
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
struct SpeciesFacet {
    scientific_name: Vec<Facet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSpeciesFacet {
    #[serde(rename(deserialize = "raw_scientificName"))]
    scientific_name: Vec<Facet>,
}

impl From<Facet> for GenusBreakdownItem {
    fn from(source: Facet) -> Self {
        GenusBreakdownItem {
            name: source.value,
            total: source.count,
        }
    }
}


#[async_trait]
impl GetFamilyStats for Solr {
    type Error = Error;

    async fn family_stats(&self, genus: &str) -> Result<FamilyStats, Error> {
        let breakdown = self.family_breakdown(genus).await?;
        Ok(FamilyStats {
            total_genera: breakdown.genera.len(),
        })
    }
}

#[async_trait]
impl GetFamilyBreakdown for Solr {
    type Error = Error;

    async fn family_breakdown(&self, family: &str) -> Result<FamilyBreakdown, Error> {
        let filter = &format!("family:{family}");

        // get all species that have a matched name. this filters
        // out records that couldn't be matched by the name-matching service
        let params = vec![
            ("q", "*:*"),
            ("rows", "0"),
            ("fq", filter),
            ("fq", r#"taxonRank:"genus""#),
            ("facet", "true"),
            ("facet.pivot", "scientificName"),
        ];

        tracing::debug!(?params);
        let (_, facets) = self.client.select_faceted::<DataRecords, GenusFacet>(&params).await?;

        Ok(FamilyBreakdown {
            genera: facets.scientific_name.into_iter().map(|s| s.into()).collect(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenusFacet {
    scientific_name: Vec<Facet>,
}

impl From<Facet> for FamilyBreakdownItem {
    fn from(source: Facet) -> Self {
        FamilyBreakdownItem {
            name: source.value,
            total: source.count,
        }
    }
}
