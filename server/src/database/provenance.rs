use arga_core::models::{Dataset, Operation};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{schema, Error, PgPool};

#[derive(Clone)]
pub struct ProvenanceProvider {
    pub pool: PgPool,
}

impl ProvenanceProvider {
    pub async fn find_by_entity_id(&self, entity_id: &str) -> Result<Vec<Operation>, Error> {
        use schema::operation_logs;
        let mut conn = self.pool.get().await?;

        let operations = operation_logs::table
            .filter(operation_logs::object_id.eq(entity_id))
            .load::<Operation>(&mut conn)
            .await?;

        Ok(operations)
    }

    pub async fn find_by_entity_id_with_dataset(
        &self,
        entity_id: &str,
    ) -> Result<Vec<(Operation, Dataset)>, Error> {
        use schema::{datasets, operation_logs};
        let mut conn = self.pool.get().await?;

        let operations = operation_logs::table
            .inner_join(datasets::table.on(datasets::id.eq(operation_logs::dataset_id)))
            .filter(operation_logs::object_id.eq(entity_id))
            .select((Operation::as_select(), Dataset::as_select()))
            .load::<(Operation, Dataset)>(&mut conn)
            .await?;

        Ok(operations)
    }
}
