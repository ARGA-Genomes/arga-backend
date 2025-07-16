use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{schema, Error, PgPool};
use crate::database::models::{AccessionEvent, CollectionEvent, Organism};


#[derive(Clone)]
pub struct OrganismProvider {
    pub pool: PgPool,
}

impl OrganismProvider {
    pub async fn find_by_id(&self, entity_id: &str) -> Result<Organism, Error> {
        use schema::organisms;
        let mut conn = self.pool.get().await?;

        let organism = organisms::table
            .filter(organisms::entity_id.eq(entity_id))
            .select(Organism::as_select())
            .get_result::<Organism>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = organism {
            return Err(Error::NotFound(entity_id.to_string()));
        }

        Ok(organism?)
    }

    pub async fn collection_events(&self, organism_entity_id: &str) -> Result<Vec<CollectionEvent>, Error> {
        use schema::{collection_events, specimens};
        let mut conn = self.pool.get().await?;

        let collections = collection_events::table
            .inner_join(specimens::table)
            .filter(specimens::organism_id.eq(organism_entity_id))
            .select(CollectionEvent::as_select())
            .load::<CollectionEvent>(&mut conn)
            .await?;

        Ok(collections)
    }

    pub async fn accession_events(&self, organism_entity_id: &str) -> Result<Vec<AccessionEvent>, Error> {
        use schema::{accession_events, specimens};
        let mut conn = self.pool.get().await?;

        let accessions = accession_events::table
            .inner_join(specimens::table)
            .filter(specimens::organism_id.eq(organism_entity_id))
            .select(AccessionEvent::as_select())
            .load::<AccessionEvent>(&mut conn)
            .await?;

        Ok(accessions)
    }
}
