use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{DnaExtract, DnaExtractionEvent};
use super::{schema, Error, PgPool};


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

    pub async fn find_by_accession(&self, record_id: &str) -> Result<Option<DnaExtract>, Error> {
        use schema::dna_extracts;
        let mut conn = self.pool.get().await?;

        let dna_extract = dna_extracts::table
            .filter(dna_extracts::record_id.eq(record_id))
            .get_result::<DnaExtract>(&mut conn)
            .await
            .optional()?;

        Ok(dna_extract)
    }

    pub async fn find_by_specimen_accession(&self, record_id: &str) -> Result<Option<DnaExtract>, Error> {
        use schema::{specimens, subsamples, dna_extracts};
        let mut conn = self.pool.get().await?;

        let extract = specimens::table
            .inner_join(subsamples::table)
            .inner_join(dna_extracts::table.on(subsamples::id.eq(dna_extracts::subsample_id)))
            .select(dna_extracts::all_columns)
            .filter(specimens::record_id.eq(record_id))
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
