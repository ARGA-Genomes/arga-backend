use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::{schema, Error, PgPool};
use crate::database::models::{Subsample, SubsampleEvent};


#[derive(Clone)]
pub struct SubsampleProvider {
    pub pool: PgPool,
}

impl SubsampleProvider {
    pub async fn find_by_id(&self, subsample_id: &Uuid) -> Result<Option<Subsample>, Error> {
        use schema::subsamples;
        let mut conn = self.pool.get().await?;

        let subsample = subsamples::table
            .filter(subsamples::id.eq(subsample_id))
            .get_result::<Subsample>(&mut conn)
            .await
            .optional()?;

        Ok(subsample)
    }

    pub async fn find_by_record_id(&self, record_id: &str) -> Result<Option<Subsample>, Error> {
        use schema::subsamples;
        let mut conn = self.pool.get().await?;

        let subsample = subsamples::table
            .filter(subsamples::record_id.eq(record_id))
            .get_result::<Subsample>(&mut conn)
            .await
            .optional()?;

        Ok(subsample)
    }

    pub async fn find_by_specimen_record_id(&self, record_id: &str) -> Result<Option<Subsample>, Error> {
        use schema::{specimens_old as specimens, subsamples};
        let mut conn = self.pool.get().await?;

        let subsample = specimens::table
            .inner_join(subsamples::table)
            .select(subsamples::all_columns)
            .filter(specimens::record_id.eq(record_id))
            .get_result::<Subsample>(&mut conn)
            .await
            .optional()?;

        Ok(subsample)
    }

    pub async fn subsample_events(&self, subsample_id: &Uuid) -> Result<Vec<SubsampleEvent>, Error> {
        use schema::subsample_events;
        let mut conn = self.pool.get().await?;

        let subsamples = subsample_events::table
            .filter(subsample_events::subsample_id.eq(subsample_id))
            .load::<SubsampleEvent>(&mut conn)
            .await?;

        Ok(subsamples)
    }
}
