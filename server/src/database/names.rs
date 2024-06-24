use arga_core::models::{Name, Taxon};
use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::PgPool;
use crate::database::schema;
use crate::http::Error;


sql_function!(fn lower(x: Text) -> Text);


#[derive(Clone)]
pub struct NameProvider {
    pub pool: PgPool,
}

impl NameProvider {
    pub async fn find_by_name_id(&self, uuid: &Uuid) -> Result<Name, Error> {
        use schema::names::dsl::*;
        let mut conn = self.pool.get().await?;

        let record = names
            .filter(id.eq(uuid))
            .order_by(scientific_name)
            .get_result::<Name>(&mut conn)
            .await?;

        Ok(record)
    }

    pub async fn find_by_canonical_name(&self, name: &str) -> Result<Vec<Name>, Error> {
        use schema::names::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = names
            .filter(lower(canonical_name).eq(name.to_lowercase()))
            .order_by(scientific_name)
            .load::<Name>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn find_by_scientific_name(&self, name: &str) -> Result<Name, Error> {
        use schema::names::dsl::*;
        let mut conn = self.pool.get().await?;

        let name = names
            .filter(scientific_name.eq(name))
            .order_by(scientific_name)
            .first::<Name>(&mut conn)
            .await?;

        Ok(name)
    }

    pub async fn taxa(&self, name_id: &Uuid) -> Result<Vec<Taxon>, Error> {
        use schema::{taxa, taxon_names};
        let mut conn = self.pool.get().await?;

        let taxa = taxa::table
            .inner_join(taxon_names::table)
            .filter(taxon_names::name_id.eq(name_id))
            .select(Taxon::as_select())
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(taxa)
    }
}
