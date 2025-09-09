use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::{DnaExtract, entity_hash};


#[derive(Clone)]
pub struct DnaExtractProvider {
    pub pool: PgPool,
}

impl DnaExtractProvider {
    pub async fn find_by_id(&self, dna_extract_id: &str) -> Result<Option<DnaExtract>, Error> {
        use schema::dna_extracts;
        let mut conn = self.pool.get().await?;

        let dna_extract = dna_extracts::table
            .filter(dna_extracts::entity_id.eq(dna_extract_id))
            .get_result::<DnaExtract>(&mut conn)
            .await
            .optional()?;

        Ok(dna_extract)
    }

    pub async fn find_by_record_id(&self, record_id: &str) -> Result<Option<DnaExtract>, Error> {
        use schema::dna_extracts;
        let mut conn = self.pool.get().await?;

        let dna_extract = dna_extracts::table
            .filter(dna_extracts::extract_id.eq(record_id))
            .get_result::<DnaExtract>(&mut conn)
            .await
            .optional()?;

        Ok(dna_extract)
    }

    pub async fn find_by_specimen_record_id(&self, record_id: &str) -> Result<Option<DnaExtract>, Error> {
        use schema::{dna_extracts, subsamples};
        let mut conn = self.pool.get().await?;

        let extract = dna_extracts::table
            .inner_join(subsamples::table)
            .select(dna_extracts::all_columns)
            .filter(subsamples::specimen_id.eq(entity_hash(record_id)))
            .get_result::<DnaExtract>(&mut conn)
            .await
            .optional()?;

        Ok(extract)
    }
}
