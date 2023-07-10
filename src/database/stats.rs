use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::stats::{GetGenusStats, GenusStats, FamilyStats, GetFamilyStats, GetGenusBreakdown, GenusBreakdown, GenusBreakdownItem};
use super::{schema, schema_gnl, sum_if, Database, Error};


#[async_trait]
impl GetGenusStats for Database {
    type Error = Error;

    async fn genus_stats(&self, genus_value: &str) -> Result<GenusStats, Error> {
        use schema_gnl::ranked_taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let (valid, total): (i64, i64) = ranked_taxa
            .select((
                sum_if(taxonomic_status.eq("accepted")),
                diesel::dsl::count_star()
            ))
            .filter(genus.eq(genus_value))
            .filter(taxon_rank.eq("species"))
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
        use schema::{names, assemblies};
        use schema_gnl::ranked_taxa;
        use diesel::dsl::count_star;

        let mut conn = self.pool.get().await?;

        let groups = assemblies::table
            .inner_join(names::table)
            .inner_join(ranked_taxa::table.on(names::id.eq(ranked_taxa::name_id)))
            .filter(ranked_taxa::genus.eq(genus_value))
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
        use schema_gnl::ranked_taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total_genera: i64 = ranked_taxa
            .filter(family.eq(family_value))
            .filter(taxon_rank.eq("genus"))
            .filter(taxonomic_status.eq("accepted"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(FamilyStats {
            // this can never be negative due to the count
            total_genera: total_genera as usize
        })
    }
}
