use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::genus::{GetGenus, Taxonomy};
use super::{Database, Error, Taxon};


#[async_trait]
impl GetGenus for Database {
    type Error = Error;

    async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use crate::schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = gnl
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
            .filter(taxon_rank.eq("genus"))
            .filter(canonical_name.eq(name))
            .first::<Taxon>(&mut conn).await?;

        Ok(Taxonomy::from(taxon))
    }
}
