use async_graphql::*;

use super::common::operation_logs::{OperationBy, SpecimenOperation};
use crate::http::{Context as State, Error};


pub struct Provenance;

#[Object]
impl Provenance {
    pub async fn specimen(&self, ctx: &Context<'_>, by: OperationBy) -> Result<Vec<SpecimenOperation>, Error> {
        let state = ctx.data::<State>().unwrap();
        SpecimenOperation::new(&state.database, by).await
    }
}
