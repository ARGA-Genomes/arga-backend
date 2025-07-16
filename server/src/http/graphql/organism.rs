use async_graphql::*;

use super::common::{AccessionEvent, CollectionEvent, OrganismDetails};
use crate::database::{models, Database};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum OrganismBy {
    EntityId(String),
}

#[derive(MergedObject)]
pub struct Organism(OrganismDetails, OrganismQuery);

impl Organism {
    pub async fn new(db: &Database, by: &OrganismBy) -> Result<Organism, Error> {
        let organism = match by {
            OrganismBy::EntityId(id) => db.organisms.find_by_id(&id).await?,
        };
        let details = organism.clone().into();
        let query = OrganismQuery { organism };
        Ok(Organism(details, query))
    }
}


struct OrganismQuery {
    organism: models::Organism,
}

#[Object]
impl OrganismQuery {
    async fn collections(&self, ctx: &Context<'_>) -> Result<Vec<CollectionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let collections = state.database.organisms.collection_events(entity_id).await?;
        Ok(collections.into_iter().map(|r| r.into()).collect())
    }

    async fn accessions(&self, ctx: &Context<'_>) -> Result<Vec<AccessionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let accessions = state.database.organisms.accession_events(entity_id).await?;
        Ok(accessions.into_iter().map(|r| r.into()).collect())
    }
}
