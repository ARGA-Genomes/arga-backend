use std::collections::HashMap;

use async_trait::async_trait;
use serde::Deserialize;

use crate::index::search::{Searchable, SearchResults, SearchFilterItem, SearchItem, GroupedSearchItem, SpeciesList, SearchSuggestion};
use super::{Ala, Error};


#[async_trait]
impl Searchable for Ala {
    type Error = Error;

    async fn filtered(&self, filters: &Vec<SearchFilterItem>) -> Result<SearchResults, Error> {
        // convert the filter items to a format that solr understands, specifically {key}:{value}
        let filters = filters.iter().map(|filter| filter_to_solr_filter(filter)).collect::<Vec<String>>();

        let mut params = vec![
            ("q", "*:*"),
            ("rows", "20"),
            ("group", "true"),
            ("group.field", "scientificName"),
        ];

        // unlike solr we cant provide multiple fq fields so instead we
        // join all the filters with an 'AND' to narrow down the search
        let joined_filter = filters.join(" AND ");
        params.push(("fq", &joined_filter));

        tracing::debug!(?params);
        let results = self.client.search::<Results>(&params).await?;

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
            ("group.field", "scientificName"),
        ];

        // having multiple `fq` params is the same as using AND
        for filter in filters.iter() {
            params.push(("fq", filter));
        }

        tracing::debug!(?params);
        let results = self.client.search::<Fields>(&params).await?;

        let mut groups = Vec::new();
        for group in results.id.groups.into_iter() {
            groups.push(GroupedSearchItem {
                key: group.group_value,
                matches: group.doclist.total,
                records: group.doclist.records.into_iter().map(|r| SearchItem::from(r)).collect::<Vec<SearchItem>>(),
            });
        }

        Ok(SpeciesList {
            total: results.id.matches,
            groups,
        })
    }

    async fn suggestions(&self, query: &str) -> Result<Vec<SearchSuggestion>, Error> {
        let items = self.client.auto::<Vec<AutocompleteItem>>(query, "TAXON", 10).await?;
        let mut uniqued = HashMap::with_capacity(5);

        for item in items {
            if uniqued.len() < 5 {
                uniqued.insert(item.guid.clone(), SearchSuggestion::from(item));
            }
        }

        // TODO: the hash creates an inconsistent sort. it doesn't look like there is
        // a way to set and get boosts so a lexical sort might be enough
        Ok(uniqued.into_values().collect())
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AutocompleteItem {
    guid: String,
    name: String,
    common_name: Option<String>,
    matched_names: Vec<String>,
}

impl From<AutocompleteItem> for SearchSuggestion {
    fn from(source: AutocompleteItem) -> Self {
        Self {
            guid: source.guid,
            species_name: source.name,
            common_name: source.common_name,
            matched: source.matched_names.join(", "),
        }
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fields {
    id: Matches,
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
    #[serde(rename(deserialize = "totalRecords"))]
    total: usize,
    #[serde(rename(deserialize = "results"))]
    records: Vec<AlaSearchItem>,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlaSearchItem {
    id: String,

    guid: Option<String>,

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

    species_group: Option<Vec<String>>,
    species_subgroup: Option<Vec<String>>,
    biome: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,
    license: Option<String>,

    recorded_by: Option<Vec<String>>,
    identified_by: Option<Vec<String>>,
}

impl From<AlaSearchItem> for SearchItem {
    fn from(source: AlaSearchItem) -> Self {
        SearchItem {
            id: source.id,
            species_uuid: source.guid,
            genomic_data_records: Some(0),
            scientific_name: source.scientific_name,
            genus: source.genus,
            subgenus: source.subgenus,
            kingdom: source.kingdom,
            phylum: source.phylum,
            family: source.family,
            class: source.class,
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
