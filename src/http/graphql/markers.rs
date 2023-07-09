use async_graphql::*;
use tracing::instrument;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models::TraceFile;
use crate::database::schema;


pub struct Markers;

#[derive(SimpleObject)]
pub struct SpeciesMarker {
    pub id: String,
    pub accession: String,
    pub material_sample_id: Option<String>,
    pub gb_acs: Option<String>,
    pub marker_code: Option<String>,
    pub nucleotide: Option<String>,
    pub recorded_by: Option<String>,
    pub version: Option<String>,
    pub basepairs: Option<i64>,
    pub type_: Option<String>,
    pub shape: Option<String>,
    pub source_url: Option<String>,
    pub fasta_url: Option<String>,
    pub extra_data: Option<serde_json::Value>,
}

impl From<models::Marker> for SpeciesMarker {
    fn from(value: models::Marker) -> Self {
        Self {
            id: value.id.to_string(),
            accession: value.accession,
            material_sample_id: value.material_sample_id,
            gb_acs: value.gb_acs,
            marker_code: value.marker_code,
            nucleotide: value.nucleotide,
            recorded_by: value.recorded_by,
            version: value.version,
            basepairs: value.basepairs,
            type_: value.type_,
            shape: value.shape,
            source_url: value.source_url,
            fasta_url: value.fasta_url,
            extra_data: value.extra_data,
        }
    }
}

#[Object]
impl Markers {
    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<Vec<SpeciesMarker>, Error> {
        let state = ctx.data::<State>().unwrap();
        let markers = state.database.markers.species(&canonical_name).await?;
        let markers = markers.into_iter().map(|m| m.into()).collect();
        Ok(markers)
    }
}
