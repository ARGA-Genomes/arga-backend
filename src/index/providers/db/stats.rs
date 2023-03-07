use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::stats::{GetGenusStats, GenusStats};
use super::{Database, Error};


#[async_trait]
impl GetGenusStats for Database {
    type Error = Error;

    async fn genus_stats(&self, genus_value: &str) -> Result<GenusStats, Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total_species: i64 = taxa
            .filter(genus.eq(genus_value))
            .filter(taxonomic_status.eq("accepted"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(GenusStats {
            total_species
        })
    }
}
