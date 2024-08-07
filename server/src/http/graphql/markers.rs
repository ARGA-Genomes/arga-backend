use async_graphql::*;
use tracing::instrument;
use uuid::Uuid;

use crate::database::models;
use crate::http::{Context as State, Error};


pub struct Markers;

#[derive(SimpleObject)]
pub struct SpeciesMarker {
    pub sequence_id: Uuid,
    pub dna_extract_id: Uuid,
    pub dataset_name: String,

    pub record_id: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accession: Option<String>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub target_gene: String,
    pub release_date: Option<String>,
}

impl From<models::Marker> for SpeciesMarker {
    fn from(value: models::Marker) -> Self {
        Self {
            sequence_id: value.sequence_id,
            dna_extract_id: value.dna_extract_id,
            dataset_name: value.dataset_name,
            record_id: value.record_id,
            latitude: value.latitude,
            longitude: value.longitude,
            accession: value.accession,
            sequenced_by: value.sequenced_by,
            material_sample_id: value.material_sample_id,
            target_gene: value.target_gene,
            release_date: value.release_date,
        }
    }
}

#[Object]
impl Markers {
    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<Vec<SpeciesMarker>, Error> {
        let state = ctx.data::<State>()?;
        let markers = state.database.markers.species(&canonical_name).await?;
        let markers = markers.into_iter().map(|m| m.into()).collect();
        Ok(markers)
    }
}
