use async_graphql::*;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::providers::db::Database;
use crate::index::species::{Taxonomy, Distribution, GenomicData, Region, Photo};
use crate::index::species::{GetSpecies, GetGenomicData, GetRegions, GetMedia};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::providers::db::models::Name as ArgaName;


pub struct Species {
    pub canonical_name: String,
    pub name: ArgaName,
}

#[Object]
impl Species {
    #[graphql(skip)]
    pub async fn new(db: &Database, canonical_name: String) -> Result<Species, Error> {
        use crate::schema::names;
        let mut conn = db.pool.get().await?;

        let name = names::table
            .filter(names::canonical_name.eq(&canonical_name))
            .filter(names::rank.eq("species"))
            .get_result::<ArgaName>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = name {
            return Err(Error::NotFound(canonical_name));
        }

        Ok(Species { canonical_name, name: name? })
    }

    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.canonical_name).await?;

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
        Regions { canonical_name: self.canonical_name.clone() }
    }

    #[instrument(skip(self, ctx))]
    async fn data(&self, ctx: &Context<'_>) -> Result<Vec<GenomicData>, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.db_provider.taxonomy(&self.canonical_name).await?;

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
        let photos = state.db_provider.photos(&self.canonical_name).await?;
        Ok(photos)
    }
}


pub struct Regions {
    canonical_name: String,
}

#[Object]
impl Regions {
    #[instrument(skip(self, ctx))]
    async fn ibra(&self, ctx: &Context<'_>) -> Result<Vec<Region>, Error> {
        let state = ctx.data::<State>().unwrap();
        let regions = state.db_provider.ibra(&self.canonical_name).await?;
        Ok(regions)
    }

    #[instrument(skip(self, ctx))]
    async fn imcra(&self, ctx: &Context<'_>) -> Result<Vec<Region>, Error> {
        let state = ctx.data::<State>().unwrap();
        let regions = state.db_provider.imcra(&self.canonical_name).await?;
        Ok(regions)
    }
}
