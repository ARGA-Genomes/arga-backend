use async_graphql::*;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::providers::db::Database;
use crate::index::species::ConservationStatus;
use crate::index::species::GetConservationStatus;
use crate::index::species::GetSpecimens;
use crate::index::species::GetTraceFiles;
use crate::index::species::GetWholeGenomes;
use crate::index::species::Specimen;
use crate::index::species::TraceFile;
use crate::index::species::WholeGenome;
use crate::index::species::{Taxonomy, Distribution, GenomicData, Region, Photo};
use crate::index::species::{GetSpecies, GetGenomicData, GetRegions, GetMedia};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::providers::db::models::Name as ArgaName;


pub struct Species {
    pub canonical_name: String,
    pub name: ArgaName,
    pub all_names: Vec<ArgaName>,
}

#[Object]
impl Species {
    #[graphql(skip)]
    pub async fn new(db: &Database, canonical_name: String) -> Result<Species, Error> {
        use crate::schema::names;
        let mut conn = db.pool.get().await?;

        let names = names::table
            .filter(names::canonical_name.eq(&canonical_name))
            // .filter(names::rank.eq("species"))
            .load::<ArgaName>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = names {
            return Err(Error::NotFound(canonical_name));
        }
        let names = names?;

        Ok(Species { canonical_name, name: names[0].clone(), all_names: names })
    }

    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.name).await?;

        Ok(taxonomy)
    }

    #[instrument(skip(self, ctx))]
    async fn distribution(&self, ctx: &Context<'_>) -> Result<Vec<Distribution>, Error> {
        let state = ctx.data::<State>().unwrap();
        let distribution = state.db_provider.distribution(&self.canonical_name).await?;

        Ok(distribution)
    }

    #[instrument(skip(self, _ctx))]
    async fn regions(&self, _ctx: &Context<'_>) -> Regions {
        Regions { name: self.name.clone() }
    }

    #[instrument(skip(self, ctx))]
    async fn data(&self, ctx: &Context<'_>) -> Result<Vec<GenomicData>, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.name).await?;

        let data = if let Some(canonical_name) = taxonomy.canonical_name {
            state.provider.genomic_data(&canonical_name).await?
        } else {
            vec![]
        };

        Ok(data)
    }

    #[instrument(skip(self, ctx))]
    async fn photos(&self, ctx: &Context<'_>) -> Result<Vec<Photo>, Error> {
        let state = ctx.data::<State>().unwrap();
        let photos = state.db_provider.photos(&self.name).await?;
        Ok(photos)
    }

    #[instrument(skip(self, ctx))]
    async fn specimens(&self, ctx: &Context<'_>) -> Result<Vec<Specimen>, Error> {
        let state = ctx.data::<State>().unwrap();
        let specimens = state.db_provider.specimens(&self.name).await?;
        Ok(specimens)
    }

    #[instrument(skip(self, ctx))]
    async fn conservation(&self, ctx: &Context<'_>) -> Result<Vec<ConservationStatus>> {
        let state = ctx.data::<State>().unwrap();

        let mut statuses = Vec::new();
        for name in &self.all_names {
            let records = state.db_provider.conservation_status(name).await?;
            statuses.extend(records);
        }

        Ok(statuses)
    }

    #[instrument(skip(self, ctx))]
    async fn whole_genomes(&self, ctx: &Context<'_>) -> Result<Vec<WholeGenome>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.provider.whole_genomes(&self.all_names).await?;
        Ok(records)
    }

    #[instrument(skip(self, ctx))]
    async fn trace_files(&self, ctx: &Context<'_>) -> Result<Vec<TraceFile>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.db_provider.trace_files(&self.all_names).await?;
        Ok(records)
    }
}


pub struct Regions {
    name: ArgaName,
}

#[Object]
impl Regions {
    #[instrument(skip(self, ctx))]
    async fn ibra(&self, ctx: &Context<'_>) -> Result<Vec<Region>, Error> {
        let state = ctx.data::<State>().unwrap();
        let regions = state.db_provider.ibra(&self.name).await?;
        Ok(regions)
    }

    #[instrument(skip(self, ctx))]
    async fn imcra(&self, ctx: &Context<'_>) -> Result<Vec<Region>, Error> {
        let state = ctx.data::<State>().unwrap();
        let regions = state.db_provider.imcra(&self.name).await?;
        Ok(regions)
    }
}
