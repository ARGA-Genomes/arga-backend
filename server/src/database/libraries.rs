use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::Library;


#[derive(Clone)]
pub struct LibraryProvider {
    pub pool: PgPool,
}

impl LibraryProvider {
    pub async fn find_by_id(&self, entity_id: &str) -> Result<Library, Error> {
        use schema::libraries;
        let mut conn = self.pool.get().await?;

        let library = libraries::table
            .filter(libraries::entity_id.eq(entity_id))
            .select(Library::as_select())
            .get_result::<Library>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = library {
            return Err(Error::NotFound(entity_id.to_string()));
        }

        Ok(library?)
    }

    pub async fn find_by_assembly_id(&self, entity_id: &str) -> Result<Vec<Library>, Error> {
        use schema::{libraries, library_assemblies};
        let mut conn = self.pool.get().await?;

        let records = library_assemblies::table
            .inner_join(libraries::table)
            .select(Library::as_select())
            .filter(library_assemblies::assembly_entity_id.eq(entity_id))
            .load::<Library>(&mut conn)
            .await?;

        Ok(records)
    }
}
