use std::collections::HashMap;

use arga_core::models::{Dataset, TaxonomicRank};
use arga_core::schema_gnl;
use async_graphql::SimpleObject;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Varchar};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use super::{schema, Error, PgPool};


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
pub struct TaxonStatNode {
    pub scientific_name: String,
    pub canonical_name: String,
    pub rank: TaxonomicRank,

    pub loci: Option<BigDecimal>,
    pub genomes: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,
    // pub children: Vec<TaxonStatNode>,
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

    /// Get taxon stats for a specific rank.
    /// This will return the stats for all taxons within a rank, as well as a specified
    /// amount of depth based on the taxons of interest.
    pub async fn taxon_tree(&self, rank: TaxonomicRank) -> Result<Vec<TaxonStatNode>, Error> {
        use schema::taxa;
        use schema_gnl::{taxa_tree, taxa_tree_stats};

        let mut conn = self.pool.get().await?;

        let root = diesel::alias!(taxa as root);

        let stats = taxa_tree_stats::table
            .inner_join(taxa::table.on(taxa::id.eq(taxa_tree_stats::id)))
            .inner_join(root.on(taxa::id.eq(taxa_tree_stats::taxon_id)))
            .inner_join(
                taxa_tree::table.on(taxa_tree::taxon_id
                    .eq(taxa_tree_stats::taxon_id)
                    .and(taxa_tree::id.eq(taxa_tree_stats::id))),
            )
            .select((
                taxa::scientific_name,
                taxa::canonical_name,
                taxa::rank,
                taxa_tree_stats::loci,
                taxa_tree_stats::genomes,
                taxa_tree_stats::specimens,
                taxa_tree_stats::other,
                taxa_tree_stats::total_genomic,
            ))
            .filter(taxa::rank.eq(rank))
            .load::<TaxonStatNode>(&mut conn)
            .await?;


        // select
        //     taxa_tree_stats.id,
        //     taxa.scientific_name,
        //     taxa.canonical_name,
        //     taxa.rank,
        //     loci,
        //     genomes,
        //     specimens,
        //     other,
        //     total_genomic
        // from taxa_tree_stats
        // inner join taxa on taxa_tree_stats.id=taxa.id
        // inner join taxa root on taxon_id=root.id
        // inner join taxa_tree on (taxa_tree.taxon_id = taxa_tree_stats.taxon_id and taxa_tree.id = taxa_tree_stats.id)
        // where root.rank='family' and root.scientific_name='Canidae'
        // order by taxa_tree_stats.id

        println!("{stats:#?}");

        // let mut ranks: HashMap<String, TaxonStatNode> = HashMap::new();
        // for stat in stats {
        //     let entry = ranks
        //         .entry(stat.kingdom.unwrap_or(stat.regnum.unwrap_or_default()))
        //         .and_modify(|node| node.children)
        //         .or_insert(stat);
        // }

        Ok(vec![])
    }
}
