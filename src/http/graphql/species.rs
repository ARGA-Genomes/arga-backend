use async_graphql::*;
use tracing::instrument;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::species::{
    ConservationStatus,
    GetConservationStatus,
    GetSpecimens,
    GetTraceFiles,
    GetWholeGenomes,
    TraceFile,
    WholeGenome,
    Taxonomy,
    GenomicData,
    Region,
    Photo,
    GetSpecies,
    GetGenomicData,
    GetRegions,
    GetMedia,
};
use crate::index::specimen::SpecimenDetails;
use crate::database::{schema, Database};
use crate::database::models::Name as ArgaName;
use super::markers::SpeciesMarker;


pub struct Species {
    pub canonical_name: String,
    pub name: ArgaName,
    pub all_names: Vec<ArgaName>,
}

#[Object]
impl Species {
    #[graphql(skip)]
    pub async fn new(db: &Database, canonical_name: String) -> Result<Species, Error> {
        use schema::names;
        let mut conn = db.pool.get().await?;

        let names = names::table
            .filter(names::canonical_name.eq(&canonical_name))
            .filter(names::rank.eq("species"))
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
        let taxonomy = state.database.taxonomy(&self.name).await?;

        Ok(taxonomy)
    }

    #[instrument(skip(self, _ctx))]
    async fn regions(&self, _ctx: &Context<'_>) -> Regions {
        Regions { name: self.name.clone() }
    }

    #[instrument(skip(self, ctx))]
    async fn data(&self, ctx: &Context<'_>) -> Result<Vec<GenomicData>, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.database.taxonomy(&self.name).await?;

        let data = if let Some(canonical_name) = taxonomy.canonical_name {
            state.solr.genomic_data(&canonical_name).await?
        } else {
            vec![]
        };

        Ok(data)
    }

    #[instrument(skip(self, ctx))]
    async fn photos(&self, ctx: &Context<'_>) -> Result<Vec<Photo>, Error> {
        let state = ctx.data::<State>().unwrap();
        let photos = state.database.photos(&self.name).await?;
        Ok(photos)
    }

    #[instrument(skip(self, ctx))]
    async fn specimens(&self, ctx: &Context<'_>) -> Result<Vec<SpecimenDetails>, Error> {
        let state = ctx.data::<State>().unwrap();
        let specimens = state.database.specimens(&self.name).await?;
        Ok(specimens)
    }

    #[instrument(skip(self, ctx))]
    async fn conservation(&self, ctx: &Context<'_>) -> Result<Vec<ConservationStatus>> {
        let state = ctx.data::<State>().unwrap();

        let mut statuses = Vec::new();
        for name in &self.all_names {
            let records = state.database.conservation_status(name).await?;
            statuses.extend(records);
        }

        Ok(statuses)
    }

    #[instrument(skip(self, ctx))]
    async fn whole_genomes(&self, ctx: &Context<'_>) -> Result<Vec<WholeGenome>, Error> {
        let state = ctx.data::<State>().unwrap();
        let mut records = state.solr.reference_genomes(&self.all_names).await?;
        let full = state.solr.full_genomes(&self.all_names).await?;
        let partial = state.solr.partial_genomes(&self.all_names).await?;

        records.extend(full);
        records.extend(partial);
        Ok(records)
    }

    #[instrument(skip(self, ctx))]
    async fn trace_files(&self, ctx: &Context<'_>) -> Result<Vec<TraceFile>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.database.trace_files(&self.all_names).await?;
        Ok(records)
    }

    async fn markers(&self, ctx: &Context<'_>) -> Result<Vec<SpeciesMarker>, Error> {
        let state = ctx.data::<State>().unwrap();
        let markers = state.database.markers.species(&self.canonical_name).await?;
        let markers = markers.into_iter().map(|m| m.into()).collect();
        Ok(markers)
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
        let regions = state.database.ibra(&self.name).await?;
        Ok(regions)
    }

    #[instrument(skip(self, ctx))]
    async fn imcra(&self, ctx: &Context<'_>) -> Result<Vec<Region>, Error> {
        let state = ctx.data::<State>().unwrap();
        let regions = state.database.imcra(&self.name).await?;
        Ok(regions)
    }
}
