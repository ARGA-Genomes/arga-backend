use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{database::{schema, models::Marker}, http::Error};

use super::PgPool;


#[derive(Clone)]
pub struct MarkerProvider {
    pub pool: PgPool,
}

impl MarkerProvider {
    pub async fn species(&self, canonical_name: &str) -> Result<Vec<Marker>, Error> {
        use schema::{markers, names};
        let mut conn = self.pool.get().await?;

        let records = markers::table
            .inner_join(names::table)
            .select(markers::all_columns)
            .filter(names::canonical_name.eq(canonical_name))
            .order_by(markers::accession)
            .load::<Marker>(&mut conn)
            .await?;

        Ok(records)
    }
}
