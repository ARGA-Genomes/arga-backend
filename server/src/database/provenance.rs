use arga_core::models::{Dataset, NomenclaturalActOperation, SpecimenOperation};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{schema, Error, PgPool};

#[derive(Clone)]
pub struct ProvenanceProvider {
    pub pool: PgPool,
}

impl ProvenanceProvider {
    pub async fn find_by_entity_id(&self, entity_id: &str) -> Result<Vec<NomenclaturalActOperation>, Error> {
        use schema::nomenclatural_act_logs;
        let mut conn = self.pool.get().await?;

        let operations = nomenclatural_act_logs::table
            .filter(nomenclatural_act_logs::entity_id.eq(entity_id))
            .load::<NomenclaturalActOperation>(&mut conn)
            .await?;

        Ok(operations)
    }

    pub async fn find_by_entity_id_with_dataset(
        &self,
        entity_id: &str,
    ) -> Result<Vec<(NomenclaturalActOperation, Dataset)>, Error> {
        use schema::{dataset_versions, datasets, nomenclatural_act_logs};
        let mut conn = self.pool.get().await?;

        let operations = nomenclatural_act_logs::table
            .inner_join(dataset_versions::table.on(dataset_versions::id.eq(nomenclatural_act_logs::dataset_version_id)))
            .inner_join(datasets::table.on(datasets::id.eq(dataset_versions::dataset_id)))
            .filter(nomenclatural_act_logs::entity_id.eq(entity_id))
            .select((NomenclaturalActOperation::as_select(), Dataset::as_select()))
            .load::<(NomenclaturalActOperation, Dataset)>(&mut conn)
            .await?;

        Ok(operations)
    }

    pub async fn find_specimen_logs_by_entity_id_with_dataset(
        &self,
        entity_id: &str,
    ) -> Result<Vec<(SpecimenOperation, Dataset)>, Error> {
        use schema::{dataset_versions, datasets, specimen_logs};
        let mut conn = self.pool.get().await?;

        let operations = specimen_logs::table
            .inner_join(dataset_versions::table.on(dataset_versions::id.eq(specimen_logs::dataset_version_id)))
            .inner_join(datasets::table.on(datasets::id.eq(dataset_versions::dataset_id)))
            .filter(specimen_logs::entity_id.eq(entity_id))
            .select((SpecimenOperation::as_select(), Dataset::as_select()))
            .load::<(SpecimenOperation, Dataset)>(&mut conn)
            .await?;

        Ok(operations)
    }
}
