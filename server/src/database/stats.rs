use std::collections::HashMap;

use arga_core::models::{Dataset, TaxonomicRank};
use arga_core::schema_gnl;
use async_graphql::SimpleObject;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use super::extensions::classification_filters::Classification;
use super::{schema, Error, PgPool};
use crate::database::extensions::classification_filters::with_classification;


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

    pub children: HashMap<String, TaxonStatNode>,
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
    pub async fn dataset(&self, name: &str) -> Result<DatasetStats, Error> {
        use schema::{datasets, indigenous_knowledge as iek, names};
        let mut conn = self.pool.get().await?;

        let dataset = datasets::table
            .filter(datasets::name.eq(&name))
            .get_result::<Dataset>(&mut conn)
            .await?;

        let total: i64 = names::table
            .left_join(iek::table.on(names::id.eq(iek::name_id)))
            .filter(iek::dataset_id.eq(dataset.id))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(DatasetStats {
            // this can never be negative due to the count
            total_species: total as usize,
            total_species_with_data: 0,
        })
    }

    pub async fn dataset_breakdown(&self, _name: &str) -> Result<DatasetBreakdown, Error> {
        let species = vec![];

        Ok(DatasetBreakdown { species })
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
        use schema::taxa;
        use schema_gnl::{taxa_tree, taxa_tree_stats};

        let mut conn = self.pool.get().await?;

        let root_id = taxa::table
            .select(taxa::id)
            .filter(with_classification(&taxon))
            .get_result::<uuid::Uuid>(&mut conn)
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
            children: HashMap::new(),
        }
    }
}
