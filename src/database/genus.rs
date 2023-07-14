use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::models::TaxonomicStatus;
use crate::http::graphql::common::Taxonomy;
use super::{schema, Error, PgPool};
use super::models::Taxon;


#[derive(Clone)]
pub struct GenusProvider {
    pub pool: PgPool,
}

impl GenusProvider {
    /// Get taxonomic information for a specific genus.
    pub async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = taxa
            .filter(genus.eq(name))
            .first::<Taxon>(&mut conn).await?;

        Ok(Taxonomy::from(taxon))
    }

    /// Get a list of valid and undescribed species descending from a genus.
    pub async fn species(&self, genus_name: &str) -> Result<Vec<Taxon>, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let species = taxa
            .filter(genus.eq(genus_name))
            .filter(status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(species)
    }
}
