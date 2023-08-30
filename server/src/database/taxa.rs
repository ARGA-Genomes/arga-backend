use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::extensions::filters::{with_taxonomy, Filter};

use super::extensions::Paginate;
use super::{schema, PgPool, PageResult};
use super::models::{Taxon, TaxonomicStatus};


#[derive(Clone)]
pub struct TaxaProvider {
    pub pool: PgPool,
}

impl TaxaProvider {
    pub async fn species(&self, filters: &Vec<Filter>, page: i64, per_page: i64) -> PageResult<Taxon> {
        use schema::taxa;
        let mut conn = self.pool.get().await?;

        let species = taxa::table
            .filter(taxa::status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(with_taxonomy(&filters).unwrap())
            .order_by(taxa::scientific_name)
            .paginate(page)
            .per_page(per_page)
            .load::<(Taxon, i64)>(&mut conn)
            .await?;

        Ok(species.into())
    }
}
