use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::{Error, PgPool, schema};
use crate::database::models::{DnaExtract, DnaExtractionEvent, entity_hash};


#[derive(Clone)]
pub struct DnaExtractProvider {
    pub pool: PgPool,
}

impl DnaExtractProvider {
    pub async fn find_by_id(&self, dna_extract_id: &Uuid) -> Result<Option<DnaExtract>, Error> {
        use schema::dna_extracts;
        let mut conn = self.pool.get().await?;

        let dna_extract = dna_extracts::table
            .filter(dna_extracts::id.eq(dna_extract_id))
            .get_result::<DnaExtract>(&mut conn)
            .await
            .optional()?;

        Ok(dna_extract)
    }

    pub async fn find_by_record_id(&self, record_id: &str) -> Result<Option<DnaExtract>, Error> {
        use schema::dna_extracts;
        let mut conn = self.pool.get().await?;

        let dna_extract = dna_extracts::table
            .filter(dna_extracts::record_id.eq(record_id))
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

    pub async fn dna_extraction_events(&self, dna_extract_id: &Uuid) -> Result<Vec<DnaExtractionEvent>, Error> {
        use schema::dna_extraction_events;
        let mut conn = self.pool.get().await?;

        let dna_extracts = dna_extraction_events::table
            .filter(dna_extraction_events::dna_extract_id.eq(dna_extract_id))
            .load::<DnaExtractionEvent>(&mut conn)
            .await?;

        Ok(dna_extracts)
    }
}
