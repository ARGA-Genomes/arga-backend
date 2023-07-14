use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::models::TaxonomicStatus;
use crate::http::graphql::common::Taxonomy;
use super::{schema, Error, PgPool};
use super::models::Taxon;


#[derive(Clone)]
pub struct ClassProvider {
    pub pool: PgPool,
}

impl ClassProvider {
    /// Get taxonomic information for a specific class.
    pub async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let mut taxon = taxa
            .filter(class.eq(name))
            .first::<Taxon>(&mut conn).await?;

        taxon.order = None;
        taxon.family = None;
        taxon.genus = None;
        Ok(Taxonomy::from(taxon))
    }

    /// Get a list of valid and undescribed species descending from a class.
    pub async fn species(&self, class_name: &str) -> Result<Vec<Taxon>, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let species = taxa
            .filter(class.eq(class_name))
            .filter(status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(species)
    }
}
