use async_graphql::*;

use tracing::instrument;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::maps::GetGeometry;


pub struct Maps {
    pub tolerance: Option<f32>,
}

#[Object]
impl Maps {
    #[instrument(skip(self, ctx))]
    async fn ibra(&self, ctx: &Context<'_>, regions: Vec<String>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let features = state.database.map_ibra(&regions, &self.tolerance).await.unwrap();
        let geojson = geojson::ser::to_feature_collection_string(&features).unwrap();
        Ok(geojson)
    }

    #[instrument(skip(self, ctx))]
    async fn imcra_provincial(&self, ctx: &Context<'_>, regions: Vec<String>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let features = state.database.map_imcra_provincial(&regions, &self.tolerance).await.unwrap();
        let geojson = geojson::ser::to_feature_collection_string(&features).unwrap();
        Ok(geojson)
    }

    #[instrument(skip(self, ctx))]
    async fn imcra_mesoscale(&self, ctx: &Context<'_>, regions: Vec<String>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let features = state.database.map_imcra_mesoscale(&regions, &self.tolerance).await.unwrap();
        let geojson = geojson::ser::to_feature_collection_string(&features).unwrap();
        Ok(geojson)
    }
}