use async_trait::async_trait;

use diesel::prelude::*;
use diesel::sql_types::{Text, Nullable};
use diesel_async::RunQueryDsl;
use tracing::instrument;

use crate::index::names::GetNames;

use super::{Database, Error};
use super::models::Name;


sql_function!(fn lower(x: Nullable<Text>) -> Nullable<Text>);


#[async_trait]
impl GetNames for Database {
    type Error = Error;

    #[instrument(skip(self))]
    async fn find_by_canonical_name(&self, name: &str) -> Result<Vec<Name>, Self::Error> {
        use crate::schema::names::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = names
            .filter(lower(canonical_name).eq(name.to_lowercase()))
            .order_by(scientific_name)
            .load::<Name>(&mut conn)
            .await?;

        Ok(records)
    }

    #[instrument(skip(self))]
    async fn find_by_scientific_name(&self, name: &str) -> Result<Name, Self::Error> {
        use crate::schema::names::dsl::*;
        let mut conn = self.pool.get().await?;

        let name = names
            .filter(scientific_name.eq(name))
            .order_by(scientific_name)
            .first::<Name>(&mut conn)
            .await?;

        Ok(name)
    }
}
