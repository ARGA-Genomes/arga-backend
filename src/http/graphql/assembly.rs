use async_graphql::*;
use tracing::instrument;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;
use crate::database::schema;
use crate::index::assembly::{AssemblyStats, BioSample, AssemblyDetails};
use crate::index::assembly::{GetAssemblyStats, GetBioSamples};
use crate::index::names::GetNames;



#[derive(MergedObject)]
pub struct Assembly(AssemblyDetails, AssemblyQuery);

impl Assembly {
    pub async fn new(db: &Database, accession: &str) -> Result<Assembly, Error> {
        let query = AssemblyQuery::new(db, accession).await?;
        Ok(Assembly(query.assembly.clone().into(), query))
    }
}


struct AssemblyQuery {
    assembly: models::Assembly,
}

#[Object]
impl AssemblyQuery {
    #[graphql(skip)]
    pub async fn new(db: &Database, accession: &str) -> Result<AssemblyQuery, Error> {
        use schema::assemblies;
        let mut conn = db.pool.get().await?;
        let assembly = assemblies::table
            .filter(assemblies::accession.eq(accession))
            .get_result::<models::Assembly>(&mut conn)
            .await?;

        Ok(AssemblyQuery {
            assembly,
        })
    }

    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
        let state = ctx.data::<State>().unwrap();
        let name = state.database.find_by_assembly_id(&self.assembly.id).await?;
        Ok(name.canonical_name)
    }

    #[instrument(skip(self))]
    async fn details(&self) -> AssemblyDetails {
        self.assembly.clone().into()
    }

    #[instrument(skip(self, ctx))]
    async fn stats(&self, ctx: &Context<'_>) -> Result<AssemblyStats, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.get_assembly_stats(&self.assembly.id).await?;
        Ok(stats)
    }

    #[instrument(skip(self, ctx))]
    async fn biosamples(&self, ctx: &Context<'_>) -> Result<Vec<BioSample>, Error> {
        let state = ctx.data::<State>().unwrap();

        let biosamples = match &self.assembly.biosample_id {
            Some(accession) => state.database.get_biosamples(accession).await?,
            None => vec![],
        };

        Ok(biosamples)
    }
}
