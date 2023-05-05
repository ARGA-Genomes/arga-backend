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
        let features = state.db_provider.map_ibra(&regions, &self.tolerance).await.unwrap();

        let geojson = geojson::ser::to_feature_collection_string(&features).unwrap();

        Ok(geojson)
    }
}
