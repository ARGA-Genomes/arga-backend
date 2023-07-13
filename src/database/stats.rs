use async_graphql::SimpleObject;
use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};

use crate::{index::stats::{GetGenusStats, GenusStats, FamilyStats, GetFamilyStats, GetGenusBreakdown, GenusBreakdown, GenusBreakdownItem, FamilyBreakdown}, database::models::TaxonomicStatus};
use super::{schema, schema_gnl, sum_if, Database, Error, PgPool};


#[async_trait]
impl GetGenusStats for Database {
    type Error = Error;

    async fn genus_stats(&self, genus_value: &str) -> Result<GenusStats, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let (valid, total): (i64, i64) = taxa
            .select((
                sum_if(status.eq(TaxonomicStatus::Valid)),
                diesel::dsl::count_star()
            ))
            .filter(genus.eq(genus_value))
            .group_by(genus)
            .get_result(&mut conn)
            .await?;

        Ok(GenusStats {
            // this can never be negative due to the count
            total_valid_species: valid as usize,
            total_species: total as usize
        })
    }
}

#[async_trait]
impl GetGenusBreakdown for Database {
    type Error = Error;

    async fn genus_breakdown(&self, genus_value: &str) -> Result<GenusBreakdown, Error> {
        use schema::{names, taxa, assemblies};
        use diesel::dsl::count_star;

        let mut conn = self.pool.get().await?;

        let groups = assemblies::table
            .inner_join(names::table)
            .inner_join(taxa::table.on(names::id.eq(taxa::name_id)))
            .filter(taxa::genus.eq(genus_value))
            .group_by(names::canonical_name)
            .select((names::canonical_name, count_star()))
            .load::<(Option<String>, i64)>(&mut conn)
            .await?;

        let mut species = Vec::new();
        for (name, total) in groups {
            if let Some(canonical_name) = name {
                species.push(GenusBreakdownItem {
                    canonical_name,
                    total: total as usize,
                })
            }
        }

        Ok(GenusBreakdown { species })
    }
}

#[async_trait]
impl GetFamilyStats for Database {
    type Error = Error;

    async fn family_stats(&self, family_value: &str) -> Result<FamilyStats, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total_genera: i64 = taxa
            .filter(family.eq(family_value))
            .filter(status.eq(TaxonomicStatus::Valid))
            .group_by(genus)
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(FamilyStats {
            // this can never be negative due to the count
            total_genera: total_genera as usize
        })
    }
}



#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStats {
    /// The total amount of families in the order
    pub total_families: usize,
    pub total_families_with_data: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBreakdown {
    pub families: Vec<BreakdownItem>,
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
    pub async fn order(&self, name: &str) -> Result<OrderStats, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa
            .filter(order.eq(name))
            .filter(status.eq(TaxonomicStatus::Valid))
            .group_by(family)
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(OrderStats {
            // this can never be negative due to the count
            total_families: total as usize,
            total_families_with_data: 0,
        })
    }

    pub async fn order_breakdown(&self, name: &str) -> Result<OrderBreakdown, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let families = taxa
            .filter(order.eq(name))
            .filter(status.eq(TaxonomicStatus::Valid))
            .group_by(family)
            .select((family, diesel::dsl::count_star()))
            .load::<BreakdownItem>(&mut conn)
            .await?;

        Ok(OrderBreakdown {
            families,
        })
    }
}
