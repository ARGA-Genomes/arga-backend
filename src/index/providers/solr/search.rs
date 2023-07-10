use async_trait::async_trait;
use serde::Deserialize;
use crate::http::graphql::search::WithRecordType;
use crate::index::lists::ListDataSummary;

use crate::index::search::{Searchable, SearchResults, SearchFilterItem, SearchItem, GroupedSearchItem, SpeciesList, SpeciesSearchItem, SpeciesSearch, SpeciesSearchResult, SpeciesSearchByCanonicalName, DNASearchByCanonicalName, FullTextSearch, FullTextSearchResult, FullTextSearchItem, GenomeSequenceItem, FullTextType};
use super::{Solr, Error};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataRecords {
    #[serde(rename(deserialize = "numFound"))]
    _total: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpeciesFacet {
    #[serde(alias = "scientificName", alias = "scientificName,dynamicProperties_ncbi_genome_rep", alias = "scientificName,dataResourceName")]
    scientific_name: Vec<Facet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSpeciesFacet {
    #[serde(alias = "raw_scientificName", alias = "raw_scientificName,dynamicProperties_ncbi_genome_rep", alias = "raw_scientificName,dataResourceName")]
    scientific_name: Vec<Facet>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FacetPivot {
    _field: String,
    value: String,
    count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Facet {
    _field: String,
    value: String,
    count: usize,
    pivot: Option<Vec<FacetPivot>>,
}

impl From<Facet> for SpeciesSearchItem {
    fn from(source: Facet) -> Self {

        let mut whole_genomes = 0;
        let mut partial_genomes = 0;
        let mut barcodes = 0;
        let mut mitogenomes = 0; //TODO: fix this once the data is ready

        match source.pivot {
            Some(pivot) => {

                for item in pivot {
                    if item._field == "dynamicProperties_ncbi_genome_rep" && item.value == "Full" {
                        whole_genomes = item.count;
                    }
                    else if item._field == "dynamicProperties_ncbi_genome_rep" && item.value == "Partial" {
                        partial_genomes = item.count
                    }
                    else if  item._field == "dataResourceName" && item.value == "BOLD - Australian records" {
                        barcodes = item.count;
                    }
                }
            },
            None => {}
        }

        Self {
            scientific_name: None,
            canonical_name: Some(source.value),
            total_records: source.count,
            total_genomic_records: None,
            data_summary: ListDataSummary {
                whole_genomes,
                partial_genomes,
                mitogenomes,
                barcodes ,
                other: source.count - whole_genomes - partial_genomes - barcodes - mitogenomes,
            },
            photo: Default::default()
        }
    }
}

#[async_trait]
impl SpeciesSearch for Solr {
    type Error = Error;

    async fn search_species(&self, query: Option<String>, filters: &Vec<SearchFilterItem>, results_type: Option<WithRecordType>) -> Result<SpeciesSearchResult, Error> {
        let _query = format!(r#"scientificName:"*{}*""#, query.unwrap_or_default());

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

        let mut extra_pivot = "";

        if let Some(value) = results_type {
            if value == WithRecordType::Genomes {
                extra_pivot = ",dynamicProperties_ncbi_genome_rep";
            }
            else if  value == WithRecordType::Organelles{
                //TODO: to fix once the data is ready
            }
            else if  value == WithRecordType::Barcodes{
                extra_pivot = ",dataResourceName";
            }
        }

        let pivot = "scientificName".to_owned() +extra_pivot;

        // first get species that have been matched by the name service
        let matched_params = [vec![
            ("fq", "taxonRank:species"),
            ("fl", "scientificName"),
            ("facet.pivot", &pivot),
        ], params.clone()].concat();
        tracing::debug!(?matched_params);
        let (_records, matched_facets) = self.client.select_faceted::<DataRecords, SpeciesFacet>(&matched_params).await?;

        let raw_pivot = "raw_scientificName".to_owned() +extra_pivot;

        // then we get all the species that could only be matched by genus
        let unmatched_params = [vec![
            ("fq", "matchType:higherMatch"),
            ("fq", "taxonRank:genus"),
            ("fl", "raw_scientificName"),
            ("facet.pivot", &raw_pivot),
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
    if filter.field.is_empty() {
        format!("{}", &filter.value)
    }
    else {
        format!("{}:{}", &filter.field, &filter.value)
    }
}



#[async_trait]
impl SpeciesSearchByCanonicalName for Solr {
    type Error = Error;

    async fn search_species_by_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Error> {
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

#[async_trait]
impl DNASearchByCanonicalName for Solr {
    type Error = Error;

    async fn search_dna_by_canonical_names(&self, names: &Vec<String>) -> Result<SpeciesSearchResult, Error> {
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
            ("fq", "contentTypes:GenomicDNA"),
            ("fq", r#"dataProviderName:"Barcode of Life""#),
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
            ("fq", "contentTypes:GenomicDNA"),
            ("fq", r#"dataProviderName:"Barcode of Life""#),
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



#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FullTextResponse {
    scientific_name: FullTextMatches,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FullTextMatches {
    groups: Vec<FullTextGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FullTextGroup {
    group_value: Option<String>,
    doclist: FullTextResults,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FullTextResults {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
    max_score: f32,
}


#[async_trait]
impl FullTextSearch for Solr {
    type Error = Error;

    async fn full_text(&self, query: &str) -> Result<FullTextSearchResult, Self::Error> {
        let base_params = vec![
            ("q", query),
            ("rows", "20"),
            ("group", "true"),
            ("group.field", "scientificName"),
            ("group.limit", "0"),
            ("fl", "score"),
        ];

        let mut params = base_params.clone();
        params.push(("fq", "dataResourceName:*RefSeq*"));
        tracing::debug!(?params);
        let reference_genomes = self.client.select::<FullTextResponse>(&params).await?;

        let mut params = base_params.clone();
        params.push(("fq", r#"dynamicProperties_ncbi_genome_rep:"Full""#));
        tracing::debug!(?params);
        let whole_genomes = self.client.select::<FullTextResponse>(&params).await?;

        let mut params = base_params.clone();
        params.push(("fq", r#"dynamicProperties_ncbi_genome_rep:"Partial""#));
        tracing::debug!(?params);
        let partial_genomes = self.client.select::<FullTextResponse>(&params).await?;

        let mut params = base_params.clone();
        params.push(("fq", r#"dataResourceName:"BPA Genomic Sequence Data""#));
        tracing::debug!(?params);
        let unknown_genomes = self.client.select::<FullTextResponse>(&params).await?;

        let mut params = base_params.clone();
        params.push(("fq", r#"dataProviderName:"Barcode of Life""#));
        tracing::debug!(?params);
        let barcodes = self.client.select::<FullTextResponse>(&params).await?;

        let mut records = Vec::new();

        for group in reference_genomes.scientific_name.groups.into_iter() {
            let item = GenomeSequenceItem {
                scientific_name: group.group_value.unwrap_or_default(),
                score: group.doclist.max_score * 8.0, // artificial boost to match taxon scores
                sequences: group.doclist.total,
                r#type: FullTextType::ReferenceGenomeSequence,
            };

            records.push(FullTextSearchItem::GenomeSequence(item));
        }
        for group in whole_genomes.scientific_name.groups.into_iter() {
            let item = GenomeSequenceItem {
                scientific_name: group.group_value.unwrap_or_default(),
                score: group.doclist.max_score * 8.0, // artificial boost to match taxon scores
                sequences: group.doclist.total,
                r#type: FullTextType::WholeGenomeSequence,
            };

            records.push(FullTextSearchItem::GenomeSequence(item));
        }
        for group in partial_genomes.scientific_name.groups.into_iter() {
            let item = GenomeSequenceItem {
                scientific_name: group.group_value.unwrap_or_default(),
                score: group.doclist.max_score * 8.0, // artificial boost to match taxon scores
                sequences: group.doclist.total,
                r#type: FullTextType::PartialGenomeSequence,
            };

            records.push(FullTextSearchItem::GenomeSequence(item));
        }
        for group in unknown_genomes.scientific_name.groups.into_iter() {
            let item = GenomeSequenceItem {
                scientific_name: group.group_value.unwrap_or_default(),
                score: group.doclist.max_score * 8.0, // artificial boost to match taxon scores
                sequences: group.doclist.total,
                r#type: FullTextType::UnknownGenomeSequence,
            };

            records.push(FullTextSearchItem::GenomeSequence(item));
        }
        for group in barcodes.scientific_name.groups.into_iter() {
            let item = GenomeSequenceItem {
                scientific_name: group.group_value.unwrap_or_default(),
                score: group.doclist.max_score * 8.0, // artificial boost to match taxon scores
                sequences: group.doclist.total,
                r#type: FullTextType::Barcode,
            };

            records.push(FullTextSearchItem::GenomeSequence(item));
        }

        Ok(FullTextSearchResult {
            records,
        })
    }
}
