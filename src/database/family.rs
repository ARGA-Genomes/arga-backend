use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::family::{GetFamily, Taxonomy};
use super::{schema_gnl, Database, Error, Taxon};


#[async_trait]
impl GetFamily for Database {
    type Error = Error;

    async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use schema_gnl::ranked_taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = ranked_taxa
            .select((
                scientific_name_authorship,
                canonical_name,
                kingdom,
                phylum,
                class,
                order,
                family,
                genus,
            ))
            .filter(taxon_rank.eq("family"))
            .filter(canonical_name.eq(name))
            .first::<Taxon>(&mut conn).await?;

        Ok(Taxonomy::from(taxon))
    }
}
