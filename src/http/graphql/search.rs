use std::collections::HashMap;

use async_graphql::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::http::Error;
use crate::http::Context as State;
use crate::index::lists::Pagination;
use crate::index::providers::search::SearchItem;


#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum WithRecordType {
    Genomes,
    Organelles,
    Barcodes
}


pub struct Search;

#[Object]
impl Search {
    async fn full_text(&self, ctx: &Context<'_>, query: String, data_type: Option<String>, pagination: Option<Pagination>) -> Result<FullTextSearchResult, Error> {
        let state = ctx.data::<State>().unwrap();

        let mut search_results: Vec<SearchItem> = Vec::new();
        let mut total;

        match data_type.as_ref().map(|s| s.as_str()) {
            Some("taxonomy") => {
                let results = state.search.species(&query, pagination)?;
                total = results.total;
                search_results.extend(results.results);
            }
            Some("genomes") => {
                let results = state.search.genomes(&query, pagination)?;
                total = results.total;
                search_results.extend(results.results);
            }
            // default to all search
            _ => {
                let results = state.search.species(&query, pagination)?;
                total = results.total;
                search_results.extend(results.results);
            }
        }

        let mut taxa: HashMap<Uuid, TaxonItem> = HashMap::new();
        let mut genomes: Vec<GenomeItem> = Vec::new();
        let mut name_ids: Vec<Uuid> = Vec::new();

        for item in search_results {
            match item {
                SearchItem::Species(item) => {
                    name_ids.push(item.name_id.clone());
                    taxa.insert(item.name_id, TaxonItem {
                        r#type: FullTextType::Taxon,
                        score: item.score,
                        status: serde_json::to_string(&item.status).unwrap(),

                        canonical_name: item.canonical_name,
                        subspecies: item.subspecies,
                        synonyms: item.synonyms,
                        common_names: item.common_names,
                        data_summary: DataSummary::default(),
                        classification: Classification {
                            kingdom: item.kingdom,
                            phylum: item.phylum,
                            class: item.class,
                            order: item.order,
                            family: item.family,
                            genus: item.genus,
                        },
                    });
                },
                SearchItem::Genome(item) => {
                    genomes.push(GenomeItem {
                        r#type: FullTextType::Genome,
                        score: item.score,
                        status: serde_json::to_string(&item.status).unwrap(),
                        canonical_name: item.canonical_name,

                        accession: item.accession,
                        genome_rep: item.genome_rep,
                        data_source: item.data_source,
                        level: item.level,
                        reference_genome: item.reference_genome,
                        release_date: item.release_date.map(|d| d.format("%d/%m/%Y").to_string()),
                    });
                },
            }
        }

        // get statistics for all the matched names
        let assembly_summaries = state.database.species.assembly_summary(&name_ids).await?;
        let marker_summaries = state.database.species.marker_summary(&name_ids).await?;

        for stat in assembly_summaries {
            taxa.entry(stat.name_id).and_modify(|item| {
                item.data_summary.reference_genomes += stat.reference_genomes;
                item.data_summary.whole_genomes += stat.whole_genomes;
                item.data_summary.partial_genomes += stat.partial_genomes;
            });
        }

        for stat in marker_summaries {
            taxa.entry(stat.name_id).and_modify(|item| {
                item.data_summary.barcodes += stat.barcodes;
            });
        }


        // collect results
        let taxa: Vec<FullTextSearchItem> = taxa.into_values().map(|v| FullTextSearchItem::Taxon(v)).collect();
        let genomes: Vec<FullTextSearchItem> = genomes.into_iter().map(|v| FullTextSearchItem::Genome(v)).collect();

        let mut records = Vec::with_capacity(taxa.len() + genomes.len());
        records.extend(taxa);
        records.extend(genomes);
        records.sort_by(|a, b| b.partial_cmp(a).unwrap());

        Ok(FullTextSearchResult { records, total })
    }
}


#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Enum)]
pub enum FullTextType {
    Taxon,
    Genome,
    Barcode,
}

#[derive(Debug, Default, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Classification {
    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
}

#[derive(Debug, Default, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct DataSummary {
    pub whole_genomes: i64,
    pub partial_genomes: i64,
    pub reference_genomes: i64,
    pub barcodes: i64,
}

#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct TaxonItem {
    pub canonical_name: Option<String>,
    pub subspecies: Vec<String>,
    pub synonyms: Vec<String>,
    pub common_names: Vec<String>,
    pub classification: Classification,
    pub data_summary: DataSummary,
    pub score: f32,
    pub r#type: FullTextType,
    pub status: String,
}


#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct GenomeItem {
    pub accession: String,
    pub canonical_name: Option<String>,
    pub genome_rep: Option<String>,
    pub data_source: Option<String>,
    pub level: Option<String>,
    pub reference_genome: bool,
    pub release_date: Option<String>,
    pub score: f32,
    pub r#type: FullTextType,
    pub status: String,
}


#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct FullTextSearchResult {
    pub records: Vec<FullTextSearchItem>,
    pub total: i32
}

#[derive(Debug, Union, Deserialize)]
pub enum FullTextSearchItem {
    Taxon(TaxonItem),
    Genome(GenomeItem)
}

impl FullTextSearchItem {
    pub fn score(&self) -> f32 {
        match self {
            FullTextSearchItem::Taxon(item) => item.score,
            FullTextSearchItem::Genome(item) => item.score,
        }
    }
}

impl PartialOrd for FullTextSearchItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

impl PartialEq for FullTextSearchItem {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}
