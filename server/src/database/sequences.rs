use arga_core::models::{SequencingRunEvent, AssemblyEvent, DepositionEvent, AnnotationEvent, TraceData};
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
    pub async fn find_by_id(&self, sequence_id: &Uuid) -> Result<Option<Sequence>, Error> {
        use schema::sequences;
        let mut conn = self.pool.get().await?;

        let sequence = sequences::table
            .filter(sequences::id.eq(sequence_id))
            .get_result::<Sequence>(&mut conn)
            .await
            .optional()?;

        Ok(sequence)
    }

    pub async fn find_by_record_id(&self, record_id: &str) -> Result<Vec<Sequence>, Error> {
        use schema::sequences;
        let mut conn = self.pool.get().await?;

        let sequence = sequences::table
            .filter(sequences::record_id.eq(record_id))
            .load::<Sequence>(&mut conn)
            .await?;

        Ok(sequence)
    }

    pub async fn find_by_specimen_record_id(&self, record_id: &str) -> Result<Vec<Sequence>, Error> {
        use schema::{specimens, subsamples, dna_extracts, sequences};
        let mut conn = self.pool.get().await?;

        let sequences = specimens::table
            .inner_join(subsamples::table)
            .inner_join(dna_extracts::table.on(subsamples::id.eq(dna_extracts::subsample_id)))
            .inner_join(sequences::table.on(dna_extracts::id.eq(sequences::dna_extract_id)))
            .select(sequences::all_columns)
            .filter(specimens::record_id.eq(record_id))
            .load::<Sequence>(&mut conn)
            .await?;

        Ok(sequences)
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

    pub async fn trace_data(&self, sequence_run_event_id: &Uuid) -> Result<TraceData, Error> {
        use schema::{sequencing_run_events, sequencing_events, deposition_events};
        let mut conn = self.pool.get().await?;

        let trace = sequencing_run_events::table
            .inner_join(sequencing_events::table)
            .inner_join(deposition_events::table.on(deposition_events::sequence_id.eq(sequencing_events::sequence_id)))
            .select((
                deposition_events::accession,
                sequencing_run_events::trace_id,
                sequencing_run_events::trace_name,
                sequencing_run_events::trace_link,
            ))
            .filter(sequencing_run_events::id.eq(sequence_run_event_id))
            .get_result::<TraceData>(&mut conn)
            .await?;

        Ok(trace)
    }
}
