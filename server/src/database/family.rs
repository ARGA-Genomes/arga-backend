use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::models::TaxonomicStatus;
use crate::http::graphql::common::Taxonomy;

use super::extensions::Paginate;
use super::{schema, Error, PgPool, PageResult};
use super::models::Taxon;


#[derive(Clone)]
pub struct FamilyProvider {
    pub pool: PgPool,
}

impl FamilyProvider {
    /// Get taxonomic information for a specific family.
    pub async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let mut taxon = taxa
            .filter(family.eq(name))
            .first::<Taxon>(&mut conn).await?;

        taxon.genus = None;
        Ok(Taxonomy::from(taxon))
    }

    /// Get a list of valid and undescribed species descending from a family.
    pub async fn species(&self, family_name: &str, page: i64) -> PageResult<Taxon> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let species = taxa
            .filter(family.eq(family_name))
            .filter(status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .order_by(scientific_name)
            .paginate(page)
            .load::<(Taxon, i64)>(&mut conn)
            .await?;

        Ok(species.into())
    }
}
