use async_graphql::*;

use super::common::{AccessionEvent, CollectionEvent, DnaExtractDetails, OrganismDetails, SubsampleDetails, Tissue};
use crate::database::{Database, models};
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

    async fn tissues(&self, ctx: &Context<'_>) -> Result<Vec<Tissue>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let tissues = state.database.organisms.tissues(entity_id).await?;
        Ok(tissues.into_iter().map(|r| r.into()).collect())
    }

    async fn subsamples(&self, ctx: &Context<'_>) -> Result<Vec<SubsampleDetails>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let subsamples = state.database.organisms.subsamples(entity_id).await?;
        Ok(subsamples.into_iter().map(|r| r.into()).collect())
    }

    async fn extractions(&self, ctx: &Context<'_>) -> Result<Vec<DnaExtractDetails>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let extractions = state.database.organisms.extractions(entity_id).await?;
        Ok(extractions.into_iter().map(|r| r.into()).collect())
    }
}
