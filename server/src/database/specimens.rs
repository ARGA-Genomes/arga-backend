use arga_core::models::Organism;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::{AccessionEvent, CollectionEvent, Specimen, entity_hash};


pub enum SpecimenEvent {
    Collection(CollectionEvent),
    Accession(AccessionEvent),
}


#[derive(Clone)]
pub struct SpecimenProvider {
    pub pool: PgPool,
}

impl SpecimenProvider {
    pub async fn find_by_id(&self, specimen_id: &str) -> Result<Specimen, Error> {
        use schema::specimens;
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .filter(specimens::entity_id.eq(specimen_id))
            .select(Specimen::as_select())
            .get_result::<Specimen>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = specimen {
            return Err(Error::NotFound(specimen_id.to_string()));
        }

        Ok(specimen?)
    }

    pub async fn find_by_record_id(&self, record_id: &str) -> Result<Specimen, Error> {
        use schema::specimens;
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .filter(specimens::entity_id.eq(entity_hash(record_id)))
            .select(Specimen::as_select())
            .get_result::<Specimen>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = specimen {
            return Err(Error::NotFound(record_id.to_string()));
        }

        Ok(specimen?)
    }

    pub async fn find_by_sequence_accession(&self, accession: &str) -> Result<Specimen, Error> {
        use schema::{deposition_events, dna_extracts, sequences, specimens, subsamples};
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .inner_join(subsamples::table)
            .inner_join(dna_extracts::table.on(subsamples::id.eq(dna_extracts::subsample_id)))
            .inner_join(sequences::table.on(dna_extracts::id.eq(sequences::dna_extract_id)))
            .inner_join(deposition_events::table.on(sequences::id.eq(deposition_events::sequence_id)))
            .select(Specimen::as_select())
            .filter(deposition_events::accession.eq(accession))
            .get_result::<Specimen>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = specimen {
            return Err(Error::NotFound(accession.to_string()));
        }

        Ok(specimen?)
    }

    pub async fn find_by_sequence_record_id(&self, record_id: &str) -> Result<Specimen, Error> {
        use schema::{dna_extracts, sequences, specimens, subsamples};
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .inner_join(subsamples::table)
            .inner_join(dna_extracts::table.on(subsamples::id.eq(dna_extracts::subsample_id)))
            .inner_join(sequences::table.on(dna_extracts::id.eq(sequences::dna_extract_id)))
            .select(Specimen::as_select())
            .filter(sequences::record_id.eq(record_id))
            .get_result::<Specimen>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = specimen {
            return Err(Error::NotFound(record_id.to_string()));
        }

        Ok(specimen?)
    }

    pub async fn organism(&self, specimen_id: &str) -> Result<Organism, Error> {
        use schema::{organisms, specimens};
        let mut conn = self.pool.get().await?;

        let organism = organisms::table
            .inner_join(specimens::table)
            .filter(specimens::entity_id.eq(specimen_id))
            .select(Organism::as_select())
            .get_result::<Organism>(&mut conn)
            .await?;

        Ok(organism)
    }

    pub async fn collection_events(&self, specimen_id: &str) -> Result<Vec<CollectionEvent>, Error> {
        use schema::collection_events;
        let mut conn = self.pool.get().await?;

        let collections = collection_events::table
            .filter(collection_events::specimen_id.eq(specimen_id))
            .select(CollectionEvent::as_select())
            .load::<CollectionEvent>(&mut conn)
            .await?;

        Ok(collections)
    }

    pub async fn accession_events(&self, specimen_id: &str) -> Result<Vec<AccessionEvent>, Error> {
        use schema::accession_events;
        let mut conn = self.pool.get().await?;

        let accessions = accession_events::table
            .filter(accession_events::specimen_id.eq(specimen_id))
            .select(AccessionEvent::as_select())
            .load::<AccessionEvent>(&mut conn)
            .await?;

        Ok(accessions)
    }
}
