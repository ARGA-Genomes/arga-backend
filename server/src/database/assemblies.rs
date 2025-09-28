use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::Assembly;


#[derive(Clone)]
pub struct AssemblyProvider {
    pub pool: PgPool,
}

impl AssemblyProvider {
    pub async fn find_by_id(&self, assembly_id: &str) -> Result<Option<Assembly>, Error> {
        use schema::assemblies;
        let mut conn = self.pool.get().await?;

        let assembly = assemblies::table
            .filter(assemblies::entity_id.eq(assembly_id))
            .get_result::<Assembly>(&mut conn)
            .await
            .optional()?;

        Ok(assembly)
    }
}
