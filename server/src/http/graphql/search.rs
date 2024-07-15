use std::collections::HashMap;

use arga_core::search::{SearchFilter, SearchItem};
use async_graphql::*;
use serde::Deserialize;
use uuid::Uuid;

use super::common::{convert_search_filters, SearchFilterItem};
use crate::http::{Context as State, Error};


#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum WithRecordType {
    Genomes,
    Organelles,
    Barcodes,
}


pub struct Search {
    filters: Vec<SearchFilter>,
}

#[Object]
impl Search {
    #[graphql(skip)]
    pub fn new(filters: Vec<SearchFilterItem>) -> Result<Search, Error> {
        Ok(Search {
            filters: convert_search_filters(filters)?,
        })
    }

    async fn full_text(
        &self,
        ctx: &Context<'_>,
        query: String,
        page: usize,
        per_page: usize,
    ) -> Result<FullTextSearchResult, Error> {
        let state = ctx.data::<State>()?;

        let (search_results, total) = state.search.filtered(&query, page, per_page, &self.filters)?;

        let mut name_ids: Vec<Uuid> = Vec::new();
        let mut taxa: HashMap<Uuid, TaxonItem> = HashMap::new();
        let mut genomes: Vec<GenomeItem> = Vec::new();
        let mut loci: Vec<LocusItem> = Vec::new();
        let mut specimens: Vec<SpecimenItem> = Vec::new();

        for item in search_results {
            match item {
                SearchItem::Species(item) => {
                    // NOTE: actually a taxa id. refactor the search indexing
                    // to make the difference clear
                    name_ids.push(item.name_id.clone());
                    taxa.insert(
                        item.name_id,
                        TaxonItem {
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
                                regnum: item.regnum,
                                division: item.division,
                                classis: item.classis,
                                ordo: item.ordo,
                                familia: item.familia,
                            },
                        },
                    );
                }
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
                        assembly_type: item.assembly_type,
                        reference_genome: item.reference_genome,
                        release_date: item.release_date.map(|d| d.format("%d/%m/%Y").to_string()),
                    });
                }
                SearchItem::Locus(item) => {
                    loci.push(LocusItem {
                        r#type: FullTextType::Locus,
                        score: item.score,
                        status: serde_json::to_string(&item.status).unwrap(),
                        canonical_name: item.canonical_name,

                        accession: item.accession,
                        locus_type: item.locus_type,
                        data_source: item.data_source,
                        voucher_status: item.voucher_status,
                        event_date: item.event_date.map(|d| d.format("%d/%m/%Y").to_string()),
                        event_location: item.event_location,
                    });
                }
                SearchItem::Specimen(item) => {
                    specimens.push(SpecimenItem {
                        r#type: FullTextType::Specimen,
                        score: item.score,
                        status: serde_json::to_string(&item.status).unwrap(),
                        canonical_name: item.canonical_name,

                        accession: item.accession,
                        data_source: item.data_source,
                        institution_code: item.institution_code,
                        collection_code: item.collection_code,
                        recorded_by: item.recorded_by,
                        identified_by: item.identified_by,
                        event_date: item.event_date.map(|d| d.format("%d/%m/%Y").to_string()),
                        event_location: item.event_location,
                    });
                }
            }
        }

        // get statistics for all the matched names
        let summaries = state.database.species.summary(&name_ids).await?;
        for stat in summaries {
            taxa.entry(stat.id).and_modify(|item| {
                item.data_summary.genomes = stat.genomes;
                item.data_summary.loci = stat.loci;
                item.data_summary.specimens = stat.specimens;
                item.data_summary.other = stat.other;
                item.data_summary.total_genomic = stat.total_genomic;
            });
        }

        // collect results
        let taxa: Vec<FullTextSearchItem> = taxa.into_values().map(|v| FullTextSearchItem::Taxon(v)).collect();
        let genomes: Vec<FullTextSearchItem> = genomes.into_iter().map(|v| FullTextSearchItem::Genome(v)).collect();
        let loci: Vec<FullTextSearchItem> = loci.into_iter().map(|v| FullTextSearchItem::Locus(v)).collect();
        let specimens: Vec<FullTextSearchItem> =
            specimens.into_iter().map(|v| FullTextSearchItem::Specimen(v)).collect();

        let mut records = Vec::with_capacity(taxa.len() + genomes.len() + loci.len());
        records.extend(taxa);
        records.extend(genomes);
        records.extend(loci);
        records.extend(specimens);
        records.sort_by(|a, b| b.partial_cmp(a).unwrap());

        Ok(FullTextSearchResult { records, total })
    }
}


#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Enum)]
pub enum FullTextType {
    Taxon,
    Genome,
    Locus,
    Specimen,
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

    pub regnum: Option<String>,
    pub division: Option<String>,
    pub classis: Option<String>,
    pub ordo: Option<String>,
    pub familia: Option<String>,
}

#[derive(Debug, Default, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct DataSummary {
    pub genomes: i64,
    pub loci: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
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
    pub data_source: Option<String>,
    pub genome_rep: Option<String>,
    pub level: Option<String>,
    pub assembly_type: Option<String>,
    pub reference_genome: bool,
    pub release_date: Option<String>,
    pub score: f32,
    pub r#type: FullTextType,
    pub status: String,
}

#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct LocusItem {
    pub accession: String,
    pub canonical_name: Option<String>,
    pub data_source: Option<String>,
    pub locus_type: Option<String>,
    pub voucher_status: Option<String>,
    pub event_date: Option<String>,
    pub event_location: Option<String>,
    pub score: f32,
    pub r#type: FullTextType,
    pub status: String,
}

#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct SpecimenItem {
    pub accession: String,
    pub canonical_name: Option<String>,
    pub data_source: Option<String>,
    pub institution_code: Option<String>,
    pub collection_code: Option<String>,
    pub recorded_by: Option<String>,
    pub identified_by: Option<String>,
    pub event_date: Option<String>,
    pub event_location: Option<String>,
    pub score: f32,
    pub r#type: FullTextType,
    pub status: String,
}


#[derive(Debug, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct FullTextSearchResult {
    pub records: Vec<FullTextSearchItem>,
    pub total: usize,
}

#[derive(Debug, Union, Deserialize)]
pub enum FullTextSearchItem {
    Taxon(TaxonItem),
    Genome(GenomeItem),
    Locus(LocusItem),
    Specimen(SpecimenItem),
}

impl FullTextSearchItem {
    pub fn score(&self) -> f32 {
        match self {
            FullTextSearchItem::Taxon(item) => item.score,
            FullTextSearchItem::Genome(item) => item.score,
            FullTextSearchItem::Locus(item) => item.score,
            FullTextSearchItem::Specimen(item) => item.score,
        }
    }

    pub fn canonical_name(&self) -> String {
        match self {
            FullTextSearchItem::Taxon(item) => item.canonical_name.clone().unwrap_or_default(),
            FullTextSearchItem::Genome(item) => item.canonical_name.clone().unwrap_or_default(),
            FullTextSearchItem::Locus(item) => item.canonical_name.clone().unwrap_or_default(),
            FullTextSearchItem::Specimen(item) => item.canonical_name.clone().unwrap_or_default(),
        }
    }
}

impl PartialOrd for FullTextSearchItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.score().partial_cmp(&other.score()) {
            Some(std::cmp::Ordering::Equal) => self.canonical_name().partial_cmp(&other.canonical_name()),
            Some(order) => Some(order),
            None => None,
        }
    }
}

impl PartialEq for FullTextSearchItem {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}
