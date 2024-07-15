use async_graphql::*;
use tracing::instrument;

use crate::http::{Context as State, Error};


pub struct Maps {
    pub tolerance: Option<f32>,
}

#[Object]
impl Maps {
    #[instrument(skip(self, ctx))]
    async fn ibra(&self, ctx: &Context<'_>, regions: Vec<String>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let features = state.database.maps.ibra(&regions, &self.tolerance).await?;
        let geojson = geojson::ser::to_feature_collection_string(&features)?;
        Ok(geojson)
    }

    #[instrument(skip(self, ctx))]
    async fn imcra_provincial(&self, ctx: &Context<'_>, regions: Vec<String>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let features = state.database.maps.imcra_provincial(&regions, &self.tolerance).await?;
        let geojson = geojson::ser::to_feature_collection_string(&features)?;
        Ok(geojson)
    }

    #[instrument(skip(self, ctx))]
    async fn imcra_mesoscale(&self, ctx: &Context<'_>, regions: Vec<String>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let features = state.database.maps.imcra_mesoscale(&regions, &self.tolerance).await?;
        let geojson = geojson::ser::to_feature_collection_string(&features)?;
        Ok(geojson)
    }
}
