use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::{schema, Database, Error, PgPool};
use crate::database::models::Specimen;


#[derive(Clone)]
pub struct SpecimenProvider {
    pub pool: PgPool,
}

impl SpecimenProvider {
    pub async fn find_by_id(&self, uuid: &Uuid) -> Result<Specimen, Error> {
        use schema::specimens;
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .filter(specimens::id.eq(uuid))
            .get_result::<Specimen>(&mut conn)
            .await?;

        Ok(specimen)
    }
}
