use async_graphql::*;
use tracing::instrument;

use super::common::{AccessionEvent, CollectionEvent, OrganismDetails};
use crate::database::{models, Database};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum SpecimenBy {
    EntityId(String),
    RecordId(String),
    SequenceRecordId(String),
    SequenceAccession(String),
}

#[derive(MergedObject)]
pub struct Specimen(SpecimenDetails, SpecimenQuery);

impl Specimen {
    pub async fn new(db: &Database, by: &SpecimenBy) -> Result<Specimen, Error> {
        let specimen = match by {
            SpecimenBy::EntityId(id) => db.specimens.find_by_id(&id).await?,
            SpecimenBy::RecordId(id) => db.specimens.find_by_record_id(&id).await?,
            SpecimenBy::SequenceRecordId(id) => db.specimens.find_by_sequence_record_id(&id).await?,
            SpecimenBy::SequenceAccession(id) => db.specimens.find_by_sequence_accession(&id).await?,
        };
        let details = specimen.clone().into();
        let query = SpecimenQuery { specimen };
        Ok(Specimen(details, query))
    }
}


struct SpecimenQuery {
    specimen: models::Specimen,
}

#[Object]
impl SpecimenQuery {
    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let name = state.database.names.find_by_name_id(&self.specimen.name_id).await?;
        Ok(name.canonical_name)
    }

    async fn organism(&self, ctx: &Context<'_>) -> Result<OrganismDetails, Error> {
        let state = ctx.data::<State>()?;
        let organism = state.database.specimens.organism(&self.specimen.entity_id).await?;
        Ok(organism.into())
    }

    async fn collections(&self, ctx: &Context<'_>) -> Result<Vec<CollectionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let collections = state.database.specimens.collection_events(specimen_id).await?;
        Ok(collections.into_iter().map(|r| r.into()).collect())
    }

    async fn accessions(&self, ctx: &Context<'_>) -> Result<Vec<AccessionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let accessions = state.database.specimens.accession_events(specimen_id).await?;
        Ok(accessions.into_iter().map(|r| r.into()).collect())
    }

    #[instrument(skip(self, ctx))]
    async fn events(&self, ctx: &Context<'_>) -> Result<SpecimenEvents, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let collections = state.database.specimens.collection_events(specimen_id).await?;
        let accessions = state.database.specimens.accession_events(specimen_id).await?;

        Ok(SpecimenEvents {
            collections: collections.into_iter().map(|r| r.into()).collect(),
            accessions: accessions.into_iter().map(|r| r.into()).collect(),
        })
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SpecimenDetails {
    pub entity_id: String,
    pub organism_id: String,
}

impl From<models::Specimen> for SpecimenDetails {
    fn from(value: models::Specimen) -> Self {
        Self {
            entity_id: value.entity_id,
            organism_id: value.organism_id,
        }
    }
}


#[derive(SimpleObject)]
pub struct SpecimenEvents {
    collections: Vec<CollectionEvent>,
    accessions: Vec<AccessionEvent>,
}
