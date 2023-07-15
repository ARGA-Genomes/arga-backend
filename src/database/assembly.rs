use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Assembly, AssemblyStats, BioSample, Taxon, TaxonomicStatus};
use super::{schema, Error, PgPool};


#[derive(Clone)]
pub struct AssemblyProvider {
    pub pool: PgPool,
}

impl AssemblyProvider {
    /// Get the full assembly details
    pub async fn details(&self, accession: &str) -> Result<Assembly, Error> {
        use schema::assemblies;
        let mut conn = self.pool.get().await?;

        let assembly = assemblies::table
            .filter(assemblies::accession.eq(accession))
            .get_result::<Assembly>(&mut conn)
            .await?;

        Ok(assembly.into())
    }

    /// Get all species that have an assembly record associated with its name
    pub async fn species(&self) -> Result<Vec<Taxon>, Error> {
        use schema::{taxa, names, assemblies};
        let mut conn = self.pool.get().await?;

        let species = names::table
            .inner_join(assemblies::table)
            .inner_join(taxa::table)
            .select(taxa::all_columns)
            .filter(taxa::status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .limit(40)
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(species)
    }

    /// Get the assembly statistics associated with the provided assembly
    ///
    /// These stats are different to the stats used elsewhere throughout the backend,
    /// specifically they are imported data and reflect statistics about the assembly
    /// itself rather than stats about the arga index
    pub async fn stats(&self, assembly_id: &Uuid) -> Result<AssemblyStats, Error> {
        use schema::assembly_stats;
        let mut conn = self.pool.get().await?;

        let stat = assembly_stats::table
            .filter(assembly_stats::assembly_id.eq(assembly_id))
            .get_result::<AssemblyStats>(&mut conn)
            .await?;

        Ok(stat)
    }

    /// Get all biosamples associated with the provided assembly
    pub async fn biosamples(&self, accession: &str) -> Result<Vec<BioSample>, Error> {
        use schema::biosamples;
        let mut conn = self.pool.get().await?;

        let records = biosamples::table
            .filter(biosamples::accession.eq(accession))
            .load::<BioSample>(&mut conn)
            .await?;

        Ok(records)
    }
}
