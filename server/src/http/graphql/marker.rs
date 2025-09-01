use async_graphql::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::database::{models, Database};
use crate::http::{Context as State, Error};


#[derive(MergedObject)]
pub struct Marker(MarkerDetails, MarkerQuery);

impl Marker {
    #[instrument(skip(db), fields(accession = %accession))]
    pub async fn new(db: &Database, accession: &str) -> Result<Marker, Error> {
        let marker = db.markers.find_by_accession(accession).await?;
        let details = marker.clone().into();
        let query = MarkerQuery { marker };
        Ok(Marker(details, query))
    }
}


struct MarkerQuery {
    marker: models::Marker,
}

#[Object]
impl MarkerQuery {
    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let name = state.database.names.find_by_name_id(&self.marker.name_id).await?;
        Ok(name.canonical_name)
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct MarkerDetails {
    pub sequence_id: Uuid,
    pub dna_extract_id: Uuid,
    pub dataset_name: String,

    pub record_id: String,
    pub accession: Option<String>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub target_gene: String,
}

impl From<models::Marker> for MarkerDetails {
    fn from(value: models::Marker) -> Self {
        Self {
            sequence_id: value.sequence_id,
            dna_extract_id: value.dna_extract_id,
            dataset_name: value.dataset_name,
            record_id: value.record_id,
            accession: value.accession,
            sequenced_by: value.sequenced_by,
            material_sample_id: value.material_sample_id,
            target_gene: value.target_gene,
        }
    }
}
