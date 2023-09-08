use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::Error;

use super::{schema, PgPool};
use super::models::{Source, Dataset};


#[derive(Clone)]
pub struct SourceProvider {
    pub pool: PgPool,
}

impl SourceProvider {
    pub async fn all_records(&self) -> Result<Vec<Source>, Error> {
        use schema::sources::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = sources
            .order_by(name)
            .load::<Source>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn datasets(&self, source: &Source) -> Result<Vec<Dataset>, Error> {
        use schema::datasets;
        let mut conn = self.pool.get().await?;

        let records = datasets::table
            .filter(datasets::source_id.eq(source.id))
            .order_by(datasets::name)
            .load::<Dataset>(&mut conn)
            .await?;

        Ok(records)
    }
}
