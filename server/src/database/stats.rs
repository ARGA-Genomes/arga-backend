use std::collections::HashMap;

use arga_core::models::{self, Dataset, TaxonomicRank};
use arga_core::schema_gnl;
use async_graphql::SimpleObject;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use super::extensions::classification_filters::Classification;
use super::extensions::filters_new::name_attributes::Attribute;
use super::{Error, PgPool, schema};
use crate::database::extensions::classification_filters::with_classification;
use crate::database::extensions::filters::with_classification as with_species_classification;
use crate::database::extensions::filters_new::stats;
use crate::database::extensions::sum_if;
use crate::database::sources::ALA_DATASET_ID;


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetStats {
    /// The total amount of species in the order
    pub total_species: usize,
    pub total_species_with_data: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetBreakdown {
    pub species: Vec<BreakdownItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Queryable)]
struct TaxonStat {
    pub path_scientific_name: String,
    pub path_canonical_name: String,

    pub scientific_name: String,
    pub canonical_name: String,
    pub rank: TaxonomicRank,

    pub loci: Option<BigDecimal>,
    pub genomes: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,
    pub species: Option<i64>,

    pub full_genomes: Option<BigDecimal>,
    pub partial_genomes: Option<BigDecimal>,
    pub complete_genomes: Option<BigDecimal>,
    pub assembly_chromosomes: Option<BigDecimal>,
    pub assembly_scaffolds: Option<BigDecimal>,
    pub assembly_contigs: Option<BigDecimal>,

    pub full_genomes_coverage: i64,
    pub complete_genomes_coverage: i64,
    pub partial_genomes_coverage: i64,
    pub assembly_chromosomes_coverage: i64,
    pub assembly_scaffolds_coverage: i64,
    pub assembly_contigs_coverage: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Queryable, Default)]
pub struct TaxonStatNode {
    pub scientific_name: String,
    pub canonical_name: String,
    pub rank: TaxonomicRank,

    pub loci: Option<BigDecimal>,
    pub genomes: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,
    pub species: Option<i64>,

    pub full_genomes: Option<BigDecimal>,
    pub partial_genomes: Option<BigDecimal>,
    pub complete_genomes: Option<BigDecimal>,
    pub assembly_chromosomes: Option<BigDecimal>,
    pub assembly_scaffolds: Option<BigDecimal>,
    pub assembly_contigs: Option<BigDecimal>,

    pub full_genomes_coverage: i64,
    pub complete_genomes_coverage: i64,
    pub partial_genomes_coverage: i64,
    pub assembly_chromosomes_coverage: i64,
    pub assembly_scaffolds_coverage: i64,
    pub assembly_contigs_coverage: i64,

    pub children: HashMap<String, TaxonStatNode>,
}


#[derive(Clone, Debug, Serialize, Deserialize, Queryable, Default)]
pub struct TaxonomicRankStat {
    pub rank: TaxonomicRank,
    pub children: i64,
    pub coverage: f32,
    pub at_least_one: i64,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownItem {
    pub name: Option<String>,
    pub total: i64,
}


#[derive(Clone)]
pub struct StatsProvider {
    pub pool: PgPool,
}

impl StatsProvider {
    pub async fn test_filter(&self) -> Result<(), Error> {
        use schema_gnl::taxa_tree_stats;

        let mut conn = self.pool.get().await?;

        let attr = Attribute::new("taxon group", "mammals".into());

        let id: Vec<models::TaxonTreeStat> = taxa_tree_stats::table
            .filter(stats::taxa_has_attribute(attr.clone()))
            .select(taxa_tree_stats::all_columns)
            .load(&mut conn)
            .await?;

        println!("{id:#?}");

        let id: i64 = taxa_tree_stats::table
            .filter(stats::taxa_has_attribute(attr))
            .select(taxa_tree_stats::all_columns)
            .count()
            .get_result(&mut conn)
            .await?;

        println!("{id:#?}");
        Ok(())
    }

    pub async fn dataset_breakdown(&self, _name: &str) -> Result<DatasetBreakdown, Error> {
        let species = vec![];

        Ok(DatasetBreakdown { species })
    }

    /// Get stats for a whole rank in the default taxonomic tree.
    /// This will group all taxa that belong to one of the specified ranks and aggregate the stats
    /// that are available in the taxa stats tree itself.
    pub async fn taxonomic_ranks(
        &self,
        taxon: Classification,
        ranks: &Vec<TaxonomicRank>,
    ) -> Result<Vec<TaxonomicRankStat>, Error> {
        use diesel::dsl::{count_star, sql};
        use diesel::sql_types::Float;
        use schema::{datasets, taxa};
        use schema_gnl::taxa_tree_stats;

        let mut conn = self.pool.get().await?;

        // get the eukaryota taxon that belongs to the default taxonomic dataset
        let eukaryota_uuid = taxa::table
            .select(taxa::id)
            .filter(with_classification(&taxon))
            .into_boxed()
            .inner_join(datasets::table.on(taxa::dataset_id.eq(datasets::id)))
            .filter(datasets::global_id.eq(ALA_DATASET_ID))
            .first::<uuid::Uuid>(&mut conn)
            .await?;


        // the taxa tree will give us all taxa nodes that descend from the root node, so we can
        // aggregate knowing that we will only get on version of the taxon
        let records = taxa_tree_stats::table
            .inner_join(taxa::table.on(taxa_tree_stats::id.eq(taxa::id)))
            .filter(taxa_tree_stats::taxon_id.eq(eukaryota_uuid))
            .filter(taxa::rank.eq_any(ranks))
            .group_by(taxa::rank)
            .select((
                taxa::rank,
                count_star(),
                sum_if(taxa_tree_stats::total_full_genomes_coverage.gt(0)),
                // FIXME: either we don't need this field or we create a type safe equivalent
                sql::<Float>("(sum(total_full_genomes_coverage::float4 / case when children = 0 then 1 else children end) / count(*))::float4").assume_not_null(),
            ))
            .load::<(TaxonomicRank, i64, i64, f32)>(&mut conn)
            .await?;

        let stats = records
            .into_iter()
            .map(|(rank, children, at_least_one, coverage)| TaxonomicRankStat {
                rank,
                children,
                coverage,
                at_least_one,
            })
            .collect();

        Ok(stats)
    }

    pub async fn complete_genomes_by_year(&self, taxon: Classification) -> Result<Vec<(i32, i64)>, Error> {
        use diesel::dsl::{count_star, sql};
        use diesel::sql_types::Integer;
        use schema::{datasets, taxon_names};
        use schema_gnl::{sequence_milestones, species};

        let mut conn = self.pool.get().await?;

        let filtered_species = species::table
            .filter(with_species_classification(&taxon))
            .select(species::id)
            .into_boxed();

        // FIXME: we are skipping lots of type checks here instead of adding a date extraction utility
        // extension or an extra derived field in the whole_genomes table
        let complete_genomes = sequence_milestones::table
            .inner_join(taxon_names::table.on(taxon_names::name_id.eq(sequence_milestones::name_id)))
            .inner_join(species::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(datasets::table.on(datasets::id.eq(species::dataset_id)))
            .filter(taxon_names::taxon_id.eq_any(filtered_species))
            .filter(sequence_milestones::representation.eq("Full"))
            .filter(datasets::global_id.eq(ALA_DATASET_ID))
            .select((
                sql::<Integer>("date_part('year', to_date(deposition_date, 'YYYY/MM/DD'))::integer"),
                count_star(),
            ))
            .group_by(sql::<Integer>("1"))
            .order_by(sql::<Integer>("1"))
            .load::<(i32, i64)>(&mut conn)
            .await?;

        Ok(complete_genomes)
    }

    /// Get stats for a specific taxon and it's decendents.
    /// This will traverse the tree from the specified root taxon and stop once it reaches
    /// the last rank in the include_ranks parameter. By only including descendents in the specified
    /// ranks the overall payload is reduced and a 'tree depth' is set.
    pub async fn taxon_tree(
        &self,
        taxon: Classification,
        include_ranks: Vec<TaxonomicRank>,
    ) -> Result<Vec<TaxonStatNode>, Error> {
        use schema::{datasets, taxa};
        use schema_gnl::{taxa_tree, taxa_tree_stats};

        let mut conn = self.pool.get().await?;

        let root_id = taxa::table
            .select(taxa::id)
            .filter(with_classification(&taxon))
            .into_boxed()
            .inner_join(datasets::table.on(taxa::dataset_id.eq(datasets::id)))
            .filter(datasets::global_id.eq(ALA_DATASET_ID))
            .order(datasets::global_id.asc())
            .first::<uuid::Uuid>(&mut conn)
            .await?;

        let path = diesel::alias!(taxa as path);
        let last_rank = include_ranks.last().unwrap_or(&TaxonomicRank::Domain);

        // this query joins the taxa tree with the taxa tree stats views in order
        // to get the stats for each taxon and build a tree out of it. we do this
        // here instead of grouping in postgres for more flexibility and simplicity
        // otherwise we would need to use jsonb_build_array due to the different types
        // and deserialize the payload.
        // rust should be fast enough to handle tree construction with maps upon request
        let records = taxa_tree::table
            .inner_join(path.on(path.field(taxa::id).eq(taxa_tree::path_id)))
            .inner_join(taxa::table.on(taxa::id.eq(taxa_tree::id)))
            .inner_join(taxa_tree_stats::table.on(taxa_tree::id.eq(taxa_tree_stats::id)))
            .select((
                path.field(taxa::scientific_name),
                path.field(taxa::canonical_name),
                taxa::scientific_name,
                taxa::canonical_name,
                taxa::rank,
                taxa_tree_stats::loci,
                taxa_tree_stats::genomes,
                taxa_tree_stats::specimens,
                taxa_tree_stats::other,
                taxa_tree_stats::total_genomic,
                taxa_tree_stats::species,
                taxa_tree_stats::full_genomes,
                taxa_tree_stats::partial_genomes,
                taxa_tree_stats::complete_genomes,
                taxa_tree_stats::assembly_chromosomes,
                taxa_tree_stats::assembly_scaffolds,
                taxa_tree_stats::assembly_contigs,
                taxa_tree_stats::total_full_genomes_coverage,
                taxa_tree_stats::total_complete_genomes_coverage,
                taxa_tree_stats::total_partial_genomes_coverage,
                taxa_tree_stats::total_assembly_chromosomes_coverage,
                taxa_tree_stats::total_assembly_scaffolds_coverage,
                taxa_tree_stats::total_assembly_contigs_coverage,
            ))
            // we only wants paths generated from a specific root node otherwise
            // we'd get the same taxon from paths with different roots since the taxa
            // tree is denormalized at all levels.
            // to get around performance issues with joining on multiple conditions we
            // opt to simply filter by rows where both the taxa_tree and taxa_tree_stats
            // are describing the same root node.
            .filter(taxa_tree::taxon_id.eq(root_id))
            .filter(taxa_tree_stats::taxon_id.eq(root_id))
            .filter(path.field(taxa::rank).eq(&last_rank))
            .filter(taxa::rank.eq_any(&include_ranks))
            // this will ensure that we iterate through the tree going down from the root node
            .order((taxa_tree::path_id, taxa_tree::depth.desc()))
            .load::<TaxonStat>(&mut conn)
            .await?;

        // paths are the leaf nodes in the tree and all nodes that have the same
        // path are in the tree order, so we build each tree path based on the path name
        // and merge them all at the end
        // eg. for a tree starting at Animalia and including ranks of phylum, class, order,
        //     you will get a map similar to:
        //         "Psittaciformes": [
        //           TaxonStatNode { scientific_name: "Chordata", ...},
        //           TaxonStatNode { scientific_name: "Aves", ...},
        //           TaxonStatNode { scientific_name: "Psittaciformes", ...},
        //         ],
        let mut paths: HashMap<String, Vec<TaxonStat>> = HashMap::new();
        for record in records {
            paths
                .entry(record.path_scientific_name.clone())
                .and_modify(|arr| arr.push(record.clone()))
                .or_insert(vec![record]);
        }

        let mut root = TaxonStatNode::default();

        for (_path_name, mut names) in paths {
            names.reverse();
            build_tree(&mut root.children, names);
        }

        Ok(root.children.into_values().collect())
    }
}


fn build_tree(tree: &mut HashMap<String, TaxonStatNode>, mut rows: Vec<TaxonStat>) {
    match rows.pop() {
        Some(child) => {
            let key = child.scientific_name.clone();
            let mut node = tree.remove(&key).unwrap_or_else(|| child.into());
            build_tree(&mut node.children, rows);
            tree.insert(key, node);
        }
        None => {}
    }
}


impl From<TaxonStat> for TaxonStatNode {
    fn from(value: TaxonStat) -> Self {
        TaxonStatNode {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            rank: value.rank,
            loci: value.loci,
            genomes: value.genomes,
            specimens: value.specimens,
            other: value.other,
            total_genomic: value.total_genomic,
            species: value.species,
            full_genomes: value.full_genomes,
            partial_genomes: value.partial_genomes,
            complete_genomes: value.complete_genomes,
            assembly_chromosomes: value.assembly_chromosomes,
            assembly_scaffolds: value.assembly_scaffolds,
            assembly_contigs: value.assembly_contigs,
            full_genomes_coverage: value.full_genomes_coverage,
            complete_genomes_coverage: value.complete_genomes_coverage,
            partial_genomes_coverage: value.partial_genomes_coverage,
            assembly_chromosomes_coverage: value.assembly_chromosomes_coverage,
            assembly_scaffolds_coverage: value.assembly_scaffolds_coverage,
            assembly_contigs_coverage: value.assembly_contigs_coverage,
            children: HashMap::new(),
        }
    }
}
