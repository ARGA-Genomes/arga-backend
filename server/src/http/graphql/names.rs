use arga_core::models;
use async_graphql::*;
use tracing::{instrument, info};
use uuid::Uuid;

use super::common::taxonomy::{NameDetails, TaxonDetails};
use crate::http::{Context as State, Error};


#[derive(MergedObject)]
pub struct Name(NameDetails, NameQuery);

impl Name {
    pub fn new(name: models::Name) -> Name {
        let query = NameQuery { name_id: name.id };
        Name(name.into(), query)
    }
}


pub struct NameQuery {
    name_id: Uuid,
}

#[Object]
impl NameQuery {
    #[instrument(skip(self, ctx), fields(name_id = %self.name_id))]
    async fn taxa(&self, ctx: &Context<'_>) -> Result<Vec<TaxonDetails>, Error> {
        info!("Fetching taxa for name");
        let state = ctx.data::<State>()?;
        let taxa = state.database.names.taxa(&self.name_id).await?;
        let taxa = taxa.into_iter().map(|t| t.into()).collect();
        Ok(taxa)
    }
}
