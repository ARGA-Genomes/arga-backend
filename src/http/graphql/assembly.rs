use async_graphql::*;
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;
use crate::database::schema;
use crate::index::assembly::{BioSample, AssemblyDetails};
use crate::index::assembly::GetBioSamples;
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
    async fn stats(&self, ctx: &Context<'_>) -> Result<AssemblyDetailsStats, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.assembly.stats(&self.assembly.id).await.unwrap_or_default();
        Ok(stats.into())
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


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssemblyDetailsStats {
    pub id: String,
    pub total_length: Option<i32>,
    pub spanned_gaps: Option<i32>,
    pub unspanned_gaps: Option<i32>,
    pub region_count: Option<i32>,
    pub scaffold_count: Option<i32>,
    pub scaffold_n50: Option<i32>,
    pub scaffold_l50: Option<i32>,
    pub scaffold_n75: Option<i32>,
    pub scaffold_n90: Option<i32>,
    pub contig_count: Option<i32>,
    pub contig_n50: Option<i32>,
    pub contig_l50: Option<i32>,
    pub total_gap_length: Option<i32>,
    pub molecule_count: Option<i32>,
    pub top_level_count: Option<i32>,
    pub component_count: Option<i32>,
    pub gc_perc: Option<i32>,
}

impl From<models::AssemblyStats> for AssemblyDetailsStats {
    fn from(value: models::AssemblyStats) -> Self {
        Self {
            id: value.id.to_string(),
            total_length: value.total_length,
            spanned_gaps: value.spanned_gaps,
            unspanned_gaps: value.unspanned_gaps,
            region_count: value.region_count,
            scaffold_count: value.scaffold_count,
            scaffold_n50: value.scaffold_n50,
            scaffold_l50: value.scaffold_l50,
            scaffold_n75: value.scaffold_n75,
            scaffold_n90: value.scaffold_n90,
            contig_count: value.contig_count,
            contig_n50: value.contig_n50,
            contig_l50: value.contig_l50,
            total_gap_length: value.total_gap_length,
            molecule_count: value.molecule_count,
            top_level_count: value.top_level_count,
            component_count: value.component_count,
            gc_perc: value.gc_perc,
        }
    }
}
