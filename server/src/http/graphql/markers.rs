use async_graphql::*;
use tracing::instrument;
use uuid::Uuid;

use crate::database::models;
use crate::http::Error;
use crate::http::Context as State;


pub struct Markers;

#[derive(SimpleObject)]
pub struct SpeciesMarker {
    pub sequence_id: Uuid,
    pub dna_extract_id: Uuid,
    pub dataset_name: String,

    pub accession: String,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub target_gene: String,
}

impl From<models::Marker> for SpeciesMarker {
    fn from(value: models::Marker) -> Self {
        Self {
            sequence_id: value.sequence_id,
            dna_extract_id: value.dna_extract_id,
            dataset_name: value.dataset_name,
            accession: value.accession,
            sequenced_by: value.sequenced_by,
            material_sample_id: value.material_sample_id,
            target_gene: value.target_gene,
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
