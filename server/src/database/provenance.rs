use arga_core::models::{Dataset, DatasetVersion, NomenclaturalActOperation, SpecimenOperation, TaxonOperation};
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
    ) -> Result<Vec<(NomenclaturalActOperation, DatasetVersion, Dataset)>, Error> {
        use schema::{dataset_versions, datasets, nomenclatural_act_logs};
        let mut conn = self.pool.get().await?;

        let operations = nomenclatural_act_logs::table
            .inner_join(dataset_versions::table.on(dataset_versions::id.eq(nomenclatural_act_logs::dataset_version_id)))
            .inner_join(datasets::table.on(datasets::id.eq(dataset_versions::dataset_id)))
            .filter(nomenclatural_act_logs::entity_id.eq(entity_id))
            .select((NomenclaturalActOperation::as_select(), DatasetVersion::as_select(), Dataset::as_select()))
            .load::<(NomenclaturalActOperation, DatasetVersion, Dataset)>(&mut conn)
            .await?;

        Ok(operations)
    }

    pub async fn find_taxon_logs_by_entity_id_with_dataset(
        &self,
        entity_id: &str,
    ) -> Result<Vec<(TaxonOperation, DatasetVersion, Dataset)>, Error> {
        use schema::{dataset_versions, datasets, taxa_logs};
        let mut conn = self.pool.get().await?;

        let operations = taxa_logs::table
            .inner_join(dataset_versions::table.on(dataset_versions::id.eq(taxa_logs::dataset_version_id)))
            .inner_join(datasets::table.on(datasets::id.eq(dataset_versions::dataset_id)))
            .filter(taxa_logs::entity_id.eq(entity_id))
            .select((TaxonOperation::as_select(), DatasetVersion::as_select(), Dataset::as_select()))
            .load::<(TaxonOperation, DatasetVersion, Dataset)>(&mut conn)
            .await?;

        Ok(operations)
    }

    pub async fn find_specimen_logs_by_entity_id_with_dataset(
        &self,
        entity_id: &str,
    ) -> Result<Vec<(SpecimenOperation, DatasetVersion, Dataset)>, Error> {
        use schema::{dataset_versions, datasets, specimen_logs};
        let mut conn = self.pool.get().await?;

        let operations = specimen_logs::table
            .inner_join(dataset_versions::table.on(dataset_versions::id.eq(specimen_logs::dataset_version_id)))
            .inner_join(datasets::table.on(datasets::id.eq(dataset_versions::dataset_id)))
            .filter(specimen_logs::entity_id.eq(entity_id))
            .select((SpecimenOperation::as_select(), DatasetVersion::as_select(), Dataset::as_select()))
            .load::<(SpecimenOperation, DatasetVersion, Dataset)>(&mut conn)
            .await?;

        Ok(operations)
    }
}
