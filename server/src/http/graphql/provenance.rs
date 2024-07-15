use async_graphql::*;

use super::common::operation_logs::{NomenclaturalActOperation, OperationBy, SpecimenOperation};
use crate::http::{Context as State, Error};


pub struct Provenance;

#[Object]
impl Provenance {
    pub async fn specimen(&self, ctx: &Context<'_>, by: OperationBy) -> Result<Vec<SpecimenOperation>, Error> {
        let state = ctx.data::<State>()?;
        SpecimenOperation::new(&state.database, by).await
    }

    pub async fn nomenclatural_act(
        &self,
        ctx: &Context<'_>,
        by: OperationBy,
    ) -> Result<Vec<NomenclaturalActOperation>, Error> {
        let state = ctx.data::<State>()?;
        NomenclaturalActOperation::new(&state.database, by).await
    }
}
