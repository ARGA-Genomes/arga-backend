use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::{Assembly, Specimen};


#[derive(Clone)]
pub struct AssemblyProvider {
    pub pool: PgPool,
}

impl AssemblyProvider {
    pub async fn find_by_id(&self, entity_id: &str) -> Result<Assembly, Error> {
        use schema::assemblies;
        let mut conn = self.pool.get().await?;

        let assembly = assemblies::table
            .filter(assemblies::entity_id.eq(entity_id))
            .select(Assembly::as_select())
            .get_result::<Assembly>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = assembly {
            return Err(Error::NotFound(entity_id.to_string()));
        }

        Ok(assembly?)
    }

    pub async fn specimens(&self, entity_id: &str) -> Result<Vec<Specimen>, Error> {
        use schema::{dna_extracts, libraries, library_assemblies, specimens, subsamples};
        let mut conn = self.pool.get().await?;

        let specimens = specimens::table
            .inner_join(subsamples::table)
            .inner_join(dna_extracts::table.on(dna_extracts::subsample_id.eq(subsamples::entity_id)))
            .inner_join(libraries::table.on(libraries::extract_id.eq(dna_extracts::entity_id)))
            .inner_join(library_assemblies::table.on(library_assemblies::library_entity_id.eq(libraries::entity_id)))
            .select(Specimen::as_select())
            .filter(library_assemblies::assembly_entity_id.eq(entity_id))
            .load::<Specimen>(&mut conn)
            .await?;

        Ok(specimens)
    }
}
