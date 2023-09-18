use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{database::{schema, schema_gnl, models::Marker}, http::Error};

use super::PgPool;


#[derive(Clone)]
pub struct MarkerProvider {
    pub pool: PgPool,
}

impl MarkerProvider {
    pub async fn find_by_accession(&self, accession: &str) -> Result<Marker, Error> {
        use schema_gnl::markers;
        let mut conn = self.pool.get().await?;

        let marker = markers::table.filter(markers::accession.eq(accession)).get_result(&mut conn).await?;
        Ok(marker)
    }

    pub async fn species(&self, canonical_name: &str) -> Result<Vec<Marker>, Error> {
        use schema::names;
        use schema_gnl::markers;
        let mut conn = self.pool.get().await?;

        let records = markers::table
            .inner_join(names::table)
            .select(markers::all_columns)
            .filter(names::canonical_name.eq(canonical_name))
            .order_by(markers::accession)
            .limit(40)
            .load::<Marker>(&mut conn)
            .await?;

        Ok(records)
    }
}
