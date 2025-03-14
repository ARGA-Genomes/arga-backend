use arga_core::models;
use async_graphql::*;
use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::common::taxonomy::TaxonomicRank;
use crate::database::stats::{BreakdownItem, TaxonStatNode, TaxonomicRankStat};
use crate::http::{Context as State, Error};


pub struct Statistics;

#[Object]
impl Statistics {
    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<SpeciesStatistics, Error> {
        let state = ctx.data::<State>()?;
        let names = state.database.names.find_by_canonical_name(&canonical_name).await?;

        if names.is_empty() {
            return Err(Error::NotFound(canonical_name));
        }

        //FIXME: endpoint no longer needed?
        // let names = names.iter().map(|name| name.id).collect();
        // let assembly_summaries = state.database.species.assembly_summary(&names).await?;
        // let marker_summaries = state.database.species.marker_summary(&names).await?;

        // combine the stats for all species matching the canonical name
        let stats = SpeciesStatistics::default();
        // for stat in assembly_summaries {
        //     stats.total += (stat.whole_genomes + stat.reference_genomes + stat.partial_genomes) as usize;
        //     stats.whole_genomes += stat.whole_genomes as usize;
        //     stats.partial_genomes += stat.partial_genomes as usize;
        // }
        // for stat in marker_summaries {
        //     stats.total += stat.barcodes as usize;
        //     stats.barcodes += stat.barcodes as usize;
        // }

        Ok(stats)
    }

    async fn taxon_breakdown(
        &self,
        ctx: &Context<'_>,
        taxon_rank: TaxonomicRank,
        taxon_canonical_name: String,
        include_ranks: Vec<TaxonomicRank>,
    ) -> Result<Vec<TaxonTreeNodeStatistics>, Error> {
        let state = ctx.data::<State>()?;
        let classification = taxon_rank.to_classification(taxon_canonical_name);
        let include_ranks = include_ranks.into_iter().map(|i| i.into()).collect();

        let tree = state.database.stats.taxon_tree(classification, include_ranks).await?;

        let mut stats: Vec<TaxonTreeNodeStatistics> = tree.into_iter().map(|i| i.into()).collect();
        stats.sort();

        Ok(stats)
    }

    #[instrument(skip(self, ctx))]
    async fn dataset(&self, ctx: &Context<'_>, name: String) -> Result<DatasetStatistics, Error> {
        let state = ctx.data::<State>()?;
        let stats = state.database.stats.dataset(&name).await?;
        let breakdown = state.database.stats.dataset_breakdown(&name).await?;

        Ok(DatasetStatistics {
            total_species: stats.total_species,
            species_with_data: stats.total_species_with_data,
            breakdown: breakdown.species,
        })
    }

    async fn taxonomic_ranks(
        &self,
        ctx: &Context<'_>,
        taxon_rank: TaxonomicRank,
        taxon_canonical_name: String,
        ranks: Vec<TaxonomicRank>,
    ) -> Result<Vec<TaxonomicRankStatistic>, Error> {
        let state = ctx.data::<State>()?;
        let classification = taxon_rank.to_classification(taxon_canonical_name);
        let ranks: Vec<models::TaxonomicRank> = ranks.into_iter().map(|r| r.into()).collect();

        let stats = state.database.stats.taxonomic_ranks(classification, &ranks).await?;
        Ok(stats.into_iter().map(|s| s.into()).collect())
    }

    async fn complete_genomes_by_year(
        &self,
        ctx: &Context<'_>,
        taxon_rank: TaxonomicRank,
        taxon_canonical_name: String,
    ) -> Result<Vec<CompleteGenomesByYearStatistic>, Error> {
        let state = ctx.data::<State>()?;
        let classification = taxon_rank.to_classification(taxon_canonical_name);
        let stats = state.database.stats.complete_genomes_by_year(classification).await?;
        let stats = stats
            .into_iter()
            .map(|(year, total)| CompleteGenomesByYearStatistic { year, total })
            .collect();
        Ok(stats)
    }
}


#[derive(Clone, Debug, Default, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesStatistics {
    /// The total amount of genomic data
    pub total: usize,
    /// The total amount of whole genomes available
    pub whole_genomes: usize,
    /// The total amount of partial genomes available
    pub partial_genomes: usize,
    /// The total amount of organelles available
    pub organelles: usize,
    /// The total amount of barcodes available
    pub barcodes: usize,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetStatistics {
    /// The total amount of species
    pub total_species: usize,
    /// The total amount of species that have data records
    pub species_with_data: usize,

    /// A breakdown of species and the amount of data for it
    pub breakdown: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, Default, SimpleObject, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxonTreeNodeStatistics {
    /// The scientific name of the taxon
    pub scientific_name: String,
    /// The canonical name of the taxon
    pub canonical_name: String,
    /// The taxonomic rank
    pub rank: TaxonomicRank,

    /// The total amount of loci available
    pub loci: Option<u64>,
    /// The total amount of genomes available
    pub genomes: Option<u64>,
    /// The total amount of specimens available
    pub specimens: Option<u64>,
    /// The total amount of data related to the taxon
    pub other: Option<u64>,
    /// The total amount of genomic data
    pub total_genomic: Option<u64>,
    /// The total amount of species belonging to the taxon
    pub species: Option<u64>,

    /// The total amount of full genomes for all species under this taxon
    pub full_genomes: Option<u64>,
    /// The total amount of partial genomes for all species under this taxon
    pub partial_genomes: Option<u64>,
    /// The total amount of complete genomes for all species under this taxon
    pub complete_genomes: Option<u64>,
    /// The total amount of chromosomes for all species under this taxon
    pub assembly_chromosomes: Option<u64>,
    /// The total amount of scaffolds for all species under this taxon
    pub assembly_scaffolds: Option<u64>,
    /// The total amount of contigs for all species under this taxon
    pub assembly_contigs: Option<u64>,

    pub full_genomes_coverage: i64,
    pub complete_genomes_coverage: i64,
    pub partial_genomes_coverage: i64,
    pub assembly_chromosomes_coverage: i64,
    pub assembly_scaffolds_coverage: i64,
    pub assembly_contigs_coverage: i64,

    /// The taxa that fall below this taxon rank
    pub children: Vec<TaxonTreeNodeStatistics>,
}

impl PartialOrd for TaxonTreeNodeStatistics {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.scientific_name.partial_cmp(&other.scientific_name)
    }
}

impl Ord for TaxonTreeNodeStatistics {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.scientific_name.cmp(&other.scientific_name)
    }
}

impl PartialEq for TaxonTreeNodeStatistics {
    fn eq(&self, other: &Self) -> bool {
        self.scientific_name == other.scientific_name
    }
}

impl From<TaxonStatNode> for TaxonTreeNodeStatistics {
    fn from(value: TaxonStatNode) -> Self {
        let mut children: Vec<TaxonTreeNodeStatistics> = value.children.into_values().map(|i| i.into()).collect();
        children.sort();

        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            rank: value.rank.into(),
            loci: value.loci.map(|v| v.to_u64().unwrap_or_default()),
            genomes: value.genomes.map(|v| v.to_u64().unwrap_or_default()),
            specimens: value.specimens.map(|v| v.to_u64().unwrap_or_default()),
            other: value.other.map(|v| v.to_u64().unwrap_or_default()),
            total_genomic: value.total_genomic.map(|v| v.to_u64().unwrap_or_default()),
            species: value.species.map(|v| v as u64),
            full_genomes: value.full_genomes.map(|v| v.to_u64().unwrap_or_default()),
            partial_genomes: value.partial_genomes.map(|v| v.to_u64().unwrap_or_default()),
            complete_genomes: value.complete_genomes.map(|v| v.to_u64().unwrap_or_default()),
            assembly_chromosomes: value.assembly_chromosomes.map(|v| v.to_u64().unwrap_or_default()),
            assembly_scaffolds: value.assembly_scaffolds.map(|v| v.to_u64().unwrap_or_default()),
            assembly_contigs: value.assembly_contigs.map(|v| v.to_u64().unwrap_or_default()),
            full_genomes_coverage: value.full_genomes_coverage,
            complete_genomes_coverage: value.complete_genomes_coverage,
            partial_genomes_coverage: value.partial_genomes_coverage,
            assembly_chromosomes_coverage: value.assembly_chromosomes_coverage,
            assembly_scaffolds_coverage: value.assembly_scaffolds_coverage,
            assembly_contigs_coverage: value.assembly_contigs_coverage,
            children,
        }
    }
}


#[derive(Clone, Debug, Default, SimpleObject, Serialize, Deserialize)]
pub struct TaxonomicRankStatistic {
    pub rank: TaxonomicRank,
    pub children: i64,
    pub coverage: f32,
    pub at_least_one: i64,
}

impl From<TaxonomicRankStat> for TaxonomicRankStatistic {
    fn from(value: TaxonomicRankStat) -> Self {
        Self {
            rank: value.rank.into(),
            children: value.children,
            coverage: value.coverage,
            at_least_one: value.at_least_one,
        }
    }
}

#[derive(Clone, Debug, Default, SimpleObject, Serialize, Deserialize)]
pub struct CompleteGenomesByYearStatistic {
    pub year: i32,
    pub total: i64,
}
