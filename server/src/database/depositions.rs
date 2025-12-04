use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::Deposition;


#[derive(Clone)]
pub struct DepositionProvider {
    pub pool: PgPool,
}

impl DepositionProvider {
    pub async fn find_by_id(&self, entity_id: &str) -> Result<Deposition, Error> {
        use schema::depositions;
        let mut conn = self.pool.get().await?;

        let deposition = depositions::table
            .filter(depositions::entity_id.eq(entity_id))
            .select(Deposition::as_select())
            .get_result::<Deposition>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = deposition {
            return Err(Error::NotFound(entity_id.to_string()));
        }

        Ok(deposition?)
    }

    pub async fn find_by_assembly_id(&self, entity_id: &str) -> Result<Vec<Deposition>, Error> {
        use schema::depositions;
        let mut conn = self.pool.get().await?;

        let records = depositions::table
            .filter(depositions::assembly_id.eq(entity_id))
            .select(Deposition::as_select())
            .load::<Deposition>(&mut conn)
            .await?;

        Ok(records)
    }
}
