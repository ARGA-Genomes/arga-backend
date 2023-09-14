use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Specimen, CollectionEvent, AccessionEvent};
use super::{schema, Error, PgPool};


pub enum SpecimenEvent {
    Collection(CollectionEvent),
    Accession(AccessionEvent),
}


#[derive(Clone)]
pub struct SpecimenProvider {
    pub pool: PgPool,
}

impl SpecimenProvider {
    pub async fn find_by_id(&self, specimen_id: &Uuid) -> Result<Specimen, Error> {
        use schema::specimens;
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .filter(specimens::id.eq(specimen_id))
            .get_result::<Specimen>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = specimen {
            return Err(Error::NotFound(specimen_id.to_string()));
        }

        Ok(specimen?)
    }

    pub async fn collection_events(&self, specimen_id: &Uuid) -> Result<Vec<CollectionEvent>, Error> {
        use schema::collection_events;
        let mut conn = self.pool.get().await?;

        let collections = collection_events::table
            .filter(collection_events::specimen_id.eq(specimen_id))
            .load::<CollectionEvent>(&mut conn)
            .await?;

        Ok(collections)
    }

    pub async fn accession_events(&self, specimen_id: &Uuid) -> Result<Vec<AccessionEvent>, Error> {
        use schema::accession_events;
        let mut conn = self.pool.get().await?;

        let accessions = accession_events::table
            .filter(accession_events::specimen_id.eq(specimen_id))
            .load::<AccessionEvent>(&mut conn)
            .await?;

        Ok(accessions)
    }
}
