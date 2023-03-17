use async_trait::async_trait;
use serde::Deserialize;

use crate::index::search::{Searchable, SearchResults, SearchFilterItem, SearchItem, GroupedSearchItem, SpeciesList, SpeciesSearchItem, SpeciesSearch, SpeciesSearchResult, SpeciesSearchByCanonicalName};
use super::{Solr, Error};


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

impl From<Facet> for SpeciesSearchItem {
    fn from(source: Facet) -> Self {
        Self {
            scientific_name: None,
            canonical_name: Some(source.value),
            total_records: source.count,
        }
    }
}

#[async_trait]
impl SpeciesSearch for Solr {
    type Error = Error;

    async fn search_species(&self, query: &str, filters: &Vec<SearchFilterItem>) -> Result<SpeciesSearchResult, Error> {
        let _query = format!(r#"scientificName:"*{query}*""#);

        // convert the filter items to a format that solr understands, specifically {key}:{value}
        let filters = filters.iter().map(|filter| filter_to_solr_filter(filter)).collect::<Vec<String>>();

        let mut params = vec![
            // ("q", query.as_str()),
            ("q", "*:*"),
            ("facet", "true"),
        ];

        // having multiple `fq` params is the same as using AND
        for filter in filters.iter() {
            params.push(("fq", filter));
        }

        // first get species that have been matched by the name service
        let matched_params = [vec![
            ("fq", "taxonRank:species"),
            ("fl", "scientificName"),
            ("facet.pivot", "scientificName"),
        ], params.clone()].concat();
        tracing::debug!(?matched_params);
        let (_records, matched_facets) = self.client.select_faceted::<DataRecords, SpeciesFacet>(&matched_params).await?;

        // then we get all the species that could only be matched by genus
        let unmatched_params = [vec![
            ("fq", "matchType:higherMatch"),
            ("fq", "taxonRank:genus"),
            ("fl", "raw_scientificName"),
            ("facet.pivot", "raw_scientificName"),
        ], params].concat();
        tracing::debug!(?unmatched_params);
        let (_records, unmatched_facets) = self.client.select_faceted::<DataRecords, RawSpeciesFacet>(&unmatched_params).await?;

        let total = matched_facets.scientific_name.len() + unmatched_facets.scientific_name.len();
        let mut records = Vec::with_capacity(total);

        // we don't worry about order here as that is for the consuming api to deal with
        for facet in matched_facets.scientific_name.into_iter() {
            records.push(SpeciesSearchItem::from(facet));
        }
        for facet in unmatched_facets.scientific_name.into_iter() {
            records.push(SpeciesSearchItem::from(facet));
        }

        Ok(SpeciesSearchResult {
            records,
        })
    }
}


#[async_trait]
impl Searchable for Solr {
    type Error = Error;

    async fn filtered(&self, filters: &Vec<SearchFilterItem>) -> Result<SearchResults, Error> {
        // convert the filter items to a format that solr understands, specifically {key}:{value}
        let filters = filters.iter().map(|filter| filter_to_solr_filter(filter)).collect::<Vec<String>>();

        let mut params = vec![
            ("q", "*:*"),
            ("rows", "20"),
        ];

        // having multiple `fq` params is the same as using AND
        for filter in filters.iter() {
            params.push(("fq", filter));
        }

        tracing::debug!(?params);
        let results = self.client.select::<Results>(&params).await?;

        Ok(SearchResults {
            total: results.total,
            // records: results.records,
            records: results.records.into_iter().map(|r| SearchItem::from(r)).collect::<Vec<SearchItem>>(),
        })
    }

    async fn species(&self, filters: &Vec<SearchFilterItem>) -> Result<SpeciesList, Error> {
         // convert the filter items to a format that solr understands, specifically {key}:{value}
        let filters = filters.iter().map(|filter| filter_to_solr_filter(filter)).collect::<Vec<String>>();

        let mut params = vec![
            ("q", "*:*"),
            ("rows", "20"),
            ("group", "true"),
            ("group.field", "species"),
        ];

        // craft a single filter by joining them all with OR since the default
        // will treat it as an AND query
        let joined_filter = filters.join(" OR ");
        params.push(("fq", &joined_filter));

        tracing::debug!(?params);
        let results = self.client.select::<Fields>(&params).await?;

        let mut groups = Vec::new();
        for group in results.species.groups.into_iter() {
            groups.push(GroupedSearchItem {
                key: group.group_value,
                matches: group.doclist.total,
                records: group.doclist.records.into_iter().map(|r| SearchItem::from(r)).collect::<Vec<SearchItem>>(),
            });
        }

        Ok(SpeciesList {
            total: results.species.matches,
            groups,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fields {
    species: Matches,
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
    doclist: Results,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Results {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
    #[serde(rename(deserialize = "docs"))]
    records: Vec<SolrSearchItem>,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SolrSearchItem {
    id: String,

    #[serde(rename(deserialize = "speciesID"))]
    species_id: Option<String>,

    /// The scientific name given to this taxon
    scientific_name: Option<String>,
    /// The taxonomic genus
    genus: Option<String>,
    /// The taxonomic sub genus
    subgenus: Option<String>,
    /// The taxonomic kingdom
    kingdom: Option<String>,
    /// The taxonomic phylum
    phylum: Option<String>,
    /// The taxonomic family
    family: Option<String>,
    /// The taxonomic class
    class: Option<String>,

    species: Option<String>,
    species_group: Option<Vec<String>>,
    species_subgroup: Option<Vec<String>>,
    biome: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,
    license: Option<String>,

    recorded_by: Option<Vec<String>>,
    identified_by: Option<Vec<String>>,
}

impl From<SolrSearchItem> for SearchItem {
    fn from(source: SolrSearchItem) -> Self {
        SearchItem {
            id: source.id,
            species_uuid: source.species_id,
            genomic_data_records: Some(0),
            scientific_name: source.scientific_name.clone(),
            canonical_name: source.scientific_name,
            genus: source.genus,
            subgenus: source.subgenus,
            kingdom: source.kingdom,
            phylum: source.phylum,
            family: source.family,
            class: source.class,
            species: source.species,
            species_group: source.species_group,
            species_subgroup: source.species_subgroup,
            biome: source.biome,
            event_date: source.event_date,
            event_time: source.event_time,
            license: source.license,
            recorded_by: source.recorded_by,
            identified_by: source.identified_by,
        }
    }
}


fn filter_to_solr_filter(filter: &SearchFilterItem) -> String {
    format!("{}:{}", &filter.field, &filter.value)
}



#[async_trait]
impl SpeciesSearchByCanonicalName for Solr {
    type Error = Error;

    async fn search_species_by_canonical_names(&self, names: Vec<String>) -> Result<SpeciesSearchResult, Error> {
        // craft a single filter by joining them all with OR since the default
        // will treat it as an AND query
        let names = names.into_iter().map(|name| format!("\"{name}\"")).collect::<Vec<String>>();
        let joined_names = names.join(" OR ");

        // first get species that have been matched by the name service
        let filters = format!("scientificName:{joined_names}");
        let matched_params = vec![
            ("q", "*:*"),
            ("facet", "true"),
            ("fq", "taxonRank:species"),
            ("fq", &filters),
            ("fl", "scientificName"),
            ("facet.pivot", "scientificName"),
        ];
        tracing::debug!(?matched_params);
        let (_records, matched_facets) = self.client.select_faceted::<DataRecords, SpeciesFacet>(&matched_params).await?;

        // then we get all the species that could only be matched by genus
        let filters = format!("raw_scientificName:{joined_names}");
        let unmatched_params = vec![
            ("q", "*:*"),
            ("facet", "true"),
            ("fq", "matchType:higherMatch"),
            ("fq", "taxonRank:genus"),
            ("fq", &filters),
            ("fl", "raw_scientificName"),
            ("facet.pivot", "raw_scientificName"),
        ];
        tracing::debug!(?unmatched_params);
        let (_records, unmatched_facets) = self.client.select_faceted::<DataRecords, RawSpeciesFacet>(&unmatched_params).await?;

        let total = matched_facets.scientific_name.len() + unmatched_facets.scientific_name.len();
        let mut records = Vec::with_capacity(total);

        // we don't worry about order here as that is for the consuming api to deal with
        for facet in matched_facets.scientific_name.into_iter() {
            records.push(SpeciesSearchItem::from(facet));
        }
        for facet in unmatched_facets.scientific_name.into_iter() {
            records.push(SpeciesSearchItem::from(facet));
        }

        Ok(SpeciesSearchResult {
            records,
        })
    }
}
