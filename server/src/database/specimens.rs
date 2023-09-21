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

    pub async fn find_by_accession(&self, accession: &str) -> Result<Specimen, Error> {
        use schema::specimens;
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .filter(specimens::accession.eq(accession))
            .get_result::<Specimen>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = specimen {
            return Err(Error::NotFound(accession.to_string()));
        }

        Ok(specimen?)
    }

    pub async fn find_by_sequence_accession(&self, accession: &str) -> Result<Specimen, Error> {
        use schema::{specimens, subsamples, dna_extracts, sequences};
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .inner_join(subsamples::table)
            .inner_join(dna_extracts::table.on(subsamples::id.eq(dna_extracts::subsample_id)))
            .inner_join(sequences::table.on(dna_extracts::id.eq(sequences::dna_extract_id)))
            .select(specimens::all_columns)
            .filter(sequences::accession.eq(accession))
            .get_result::<Specimen>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = specimen {
            return Err(Error::NotFound(accession.to_string()));
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
