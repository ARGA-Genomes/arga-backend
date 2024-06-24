use arga_core::models;
use async_graphql::*;
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
    async fn taxa(&self, ctx: &Context<'_>) -> Result<Vec<TaxonDetails>, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxa = state.database.names.taxa(&self.name_id).await?;
        let taxa = taxa.into_iter().map(|t| t.into()).collect();
        Ok(taxa)
    }
}
