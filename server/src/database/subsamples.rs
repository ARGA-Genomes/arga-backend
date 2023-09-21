use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Subsample, SubsampleEvent};
use super::{schema, Error, PgPool};


#[derive(Clone)]
pub struct SubsampleProvider {
    pub pool: PgPool,
}

impl SubsampleProvider {
    pub async fn find_by_id(&self, subsample_id: &Uuid) -> Result<Subsample, Error> {
        use schema::subsamples;
        let mut conn = self.pool.get().await?;

        let subsample = subsamples::table
            .filter(subsamples::id.eq(subsample_id))
            .get_result::<Subsample>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = subsample {
            return Err(Error::NotFound(subsample_id.to_string()));
        }

        Ok(subsample?)
    }

    pub async fn find_by_accession(&self, accession: &str) -> Result<Subsample, Error> {
        use schema::subsamples;
        let mut conn = self.pool.get().await?;

        let subsample = subsamples::table
            .filter(subsamples::accession.eq(accession))
            .get_result::<Subsample>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = subsample {
            return Err(Error::NotFound(accession.to_string()));
        }

        Ok(subsample?)
    }

    pub async fn find_by_specimen_accession(&self, accession: &str) -> Result<Subsample, Error> {
        use schema::{specimens, subsamples};
        let mut conn = self.pool.get().await?;

        let subsample = specimens::table
            .inner_join(subsamples::table)
            .select(subsamples::all_columns)
            .filter(specimens::accession.eq(accession))
            .get_result::<Subsample>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = subsample {
            return Err(Error::NotFound(accession.to_string()));
        }

        Ok(subsample?)
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
