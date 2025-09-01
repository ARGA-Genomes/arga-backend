use async_graphql::*;
use tracing::{instrument, info};

use super::common::operation_logs::{NomenclaturalActOperation, OperationBy, SpecimenOperation, TaxonOperation};
use crate::http::{Context as State, Error};


pub struct Provenance;

#[Object]
impl Provenance {
    #[instrument(skip(self, ctx), fields(operation_by = ?by))]
    pub async fn specimen(&self, ctx: &Context<'_>, by: OperationBy) -> Result<Vec<SpecimenOperation>, Error> {
        info!("Fetching specimen provenance");
        let state = ctx.data::<State>()?;
        SpecimenOperation::new(&state.database, by).await
    }

    #[instrument(skip(self, ctx), fields(operation_by = ?by))]
    pub async fn taxon(&self, ctx: &Context<'_>, by: OperationBy) -> Result<Vec<TaxonOperation>, Error> {
        info!("Fetching taxon provenance");
        let state = ctx.data::<State>()?;
        TaxonOperation::new(&state.database, by).await
    }

    #[instrument(skip(self, ctx), fields(operation_by = ?by))]
    pub async fn nomenclatural_act(
        &self,
        ctx: &Context<'_>,
        by: OperationBy,
    ) -> Result<Vec<NomenclaturalActOperation>, Error> {
        info!("Fetching nomenclatural act provenance");
        let state = ctx.data::<State>()?;
        NomenclaturalActOperation::new(&state.database, by).await
    }
}
