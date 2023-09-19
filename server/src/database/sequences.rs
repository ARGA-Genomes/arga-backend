use arga_core::models::{SequencingRunEvent, AssemblyEvent, DepositionEvent, AnnotationEvent};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Sequence, SequencingEvent};
use super::{schema, Error, PgPool};


#[derive(Clone)]
pub struct SequenceProvider {
    pub pool: PgPool,
}

impl SequenceProvider {
    pub async fn find_by_id(&self, sequence_id: &Uuid) -> Result<Sequence, Error> {
        use schema::sequences;
        let mut conn = self.pool.get().await?;

        let sequence = sequences::table
            .filter(sequences::id.eq(sequence_id))
            .get_result::<Sequence>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = sequence {
            return Err(Error::NotFound(sequence_id.to_string()));
        }

        Ok(sequence?)
    }

    pub async fn sequencing_events(&self, sequence_id: &Uuid) -> Result<Vec<SequencingEvent>, Error> {
        use schema::sequencing_events;
        let mut conn = self.pool.get().await?;

        let events = sequencing_events::table
            .filter(sequencing_events::sequence_id.eq(sequence_id))
            .load::<SequencingEvent>(&mut conn)
            .await?;

        Ok(events)
    }

    pub async fn sequencing_run_events(&self, sequence_id: &Uuid) -> Result<Vec<SequencingRunEvent>, Error> {
        use schema::{sequencing_events, sequencing_run_events};
        let mut conn = self.pool.get().await?;

        let events = sequencing_run_events::table
            .inner_join(sequencing_events::table)
            .filter(sequencing_events::sequence_id.eq(sequence_id))
            .select(sequencing_run_events::all_columns)
            .load::<SequencingRunEvent>(&mut conn)
            .await?;

        Ok(events)
    }

    pub async fn assembly_events(&self, sequence_id: &Uuid) -> Result<Vec<AssemblyEvent>, Error> {
        use schema::assembly_events;
        let mut conn = self.pool.get().await?;

        let events = assembly_events::table
            .filter(assembly_events::sequence_id.eq(sequence_id))
            .load::<AssemblyEvent>(&mut conn)
            .await?;

        Ok(events)
    }

    pub async fn annotation_events(&self, sequence_id: &Uuid) -> Result<Vec<AnnotationEvent>, Error> {
        use schema::annotation_events;
        let mut conn = self.pool.get().await?;

        let events = annotation_events::table
            .filter(annotation_events::sequence_id.eq(sequence_id))
            .load::<AnnotationEvent>(&mut conn)
            .await?;

        Ok(events)
    }

    pub async fn data_deposition_events(&self, sequence_id: &Uuid) -> Result<Vec<DepositionEvent>, Error> {
        use schema::deposition_events;
        let mut conn = self.pool.get().await?;

        let events = deposition_events::table
            .filter(deposition_events::sequence_id.eq(sequence_id))
            .load::<DepositionEvent>(&mut conn)
            .await?;

        Ok(events)
    }
}