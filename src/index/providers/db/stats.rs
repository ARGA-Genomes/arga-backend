use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::stats::{GetGenusStats, GenusStats, FamilyStats, GetFamilyStats};
use super::{Database, Error};


#[async_trait]
impl GetGenusStats for Database {
    type Error = Error;

    async fn genus_stats(&self, genus_value: &str) -> Result<GenusStats, Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total_species: i64 = taxa
            .filter(genus.eq(genus_value))
            .filter(taxon_rank.eq("species"))
            .filter(taxonomic_status.eq("accepted"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(GenusStats {
            // this can never be negative due to the count
            total_species: total_species as usize
        })
    }
}

#[async_trait]
impl GetFamilyStats for Database {
    type Error = Error;

    async fn family_stats(&self, family_value: &str) -> Result<FamilyStats, Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total_genera: i64 = taxa
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
