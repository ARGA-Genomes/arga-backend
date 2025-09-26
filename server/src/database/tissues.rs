use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::Tissue;


#[derive(Clone)]
pub struct TissueProvider {
    pub pool: PgPool,
}

impl TissueProvider {
    pub async fn find_by_id(&self, tissue_id: &str) -> Result<Option<Tissue>, Error> {
        use schema::tissues;
        let mut conn = self.pool.get().await?;

        let tissue = tissues::table
            .select(Tissue::as_select())
            .filter(tissues::entity_id.eq(tissue_id))
            .get_result::<Tissue>(&mut conn)
            .await
            .optional()?;

        Ok(tissue)
    }
}
