use async_graphql::*;

use super::collection::Collection;
use super::common::{OrganismDetails, Publication};
use super::dna_extract::DnaExtract;
use super::registration::Registration;
use super::subsample::Subsample;
use super::tissue::Tissue;
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
    async fn collections(&self, ctx: &Context<'_>) -> Result<Vec<Collection>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let collections = state.database.organisms.collections(entity_id).await?;
        Ok(collections.into_iter().map(|r| Collection::from_record(r)).collect())
    }

    async fn registrations(&self, ctx: &Context<'_>) -> Result<Vec<Registration>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let records = state.database.organisms.registrations(entity_id).await?;
        Ok(records.into_iter().map(|r| Registration::from_record(r)).collect())
    }

    async fn tissues(&self, ctx: &Context<'_>) -> Result<Vec<Tissue>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let records = state.database.organisms.tissues(entity_id).await?;
        Ok(records.into_iter().map(|r| Tissue::from_record(r)).collect())
    }

    async fn subsamples(&self, ctx: &Context<'_>) -> Result<Vec<Subsample>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let records = state.database.organisms.subsamples(entity_id).await?;
        Ok(records.into_iter().map(|r| Subsample::from_record(r)).collect())
    }

    async fn extractions(&self, ctx: &Context<'_>) -> Result<Vec<DnaExtract>, Error> {
        let state = ctx.data::<State>()?;
        let entity_id = &self.organism.entity_id;
        let records = state.database.organisms.extractions(entity_id).await?;
        Ok(records.into_iter().map(|r| DnaExtract::from_record(r)).collect())
    }

    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;

        let publication = match &self.organism.publication_id {
            None => None,
            Some(publication_id) => Some(state.database.publications.find_by_id(publication_id).await?.into()),
        };

        Ok(publication)
    }
}
