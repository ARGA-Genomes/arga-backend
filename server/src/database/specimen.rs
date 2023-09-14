use std::collections::HashMap;

use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Specimen, Organism, Event, CollectionEvent, SequencingEvent, SequencingRunEvent};
use crate::index::specimen::{self, GetSpecimen, GetSpecimenEvents, EventDetails};
use super::{schema, Database, Error};


#[async_trait]
impl GetSpecimen for Database {
    type Error = Error;

    async fn get_specimen(&self, specimen_id: &Uuid) -> Result<specimen::SpecimenDetails, Self::Error> {
        use schema::specimens;
        let mut conn = self.pool.get().await?;

        let specimen = specimens::table
            .filter(specimens::id.eq(specimen_id))
            .get_result::<Specimen>(&mut conn)
            .await?;

        Ok(specimen.into())
    }
}

impl From<Specimen> for specimen::SpecimenDetails {
    fn from(value: Specimen) -> Self {
        Self {
            id: value.id.to_string(),
            type_status: value.type_status,
            institution_name: value.institution_name,
            institution_code: value.institution_code,
            collection_code: value.collection_code,
            material_sample_id: value.material_sample_id,
            recorded_by: value.recorded_by,
            organism_id: value.organism_id,
            locality: value.locality,
            latitude: value.latitude,
            longitude: value.longitude,
            details: value.details,
            remarks: value.remarks,
        }
    }
}


impl From<Organism> for specimen::Organism {
    fn from(value: Organism) -> Self {
        Self {
            id: value.id.to_string(),
            organism_id: value.organism_id,
            organism_name: value.organism_name,
            organism_scope: value.organism_scope,
            associated_organisms: value.associated_organisms,
            previous_identifications: value.previous_identifications,
            remarks: value.remarks,
        }
    }
}


#[async_trait]
impl GetSpecimenEvents for Database {
    type Error = Error;

    async fn get_specimen_events(&self, specimen_id: &Uuid) -> Result<Vec<specimen::Event>, Self::Error> {
        use schema::{events, collection_events, subsamples, dna_extracts, sequences, sequencing_events, sequencing_run_events};
        let mut conn = self.pool.get().await?;

        let mut event_map: HashMap<Uuid, Vec<EventDetails>> = HashMap::new();

        // get all collection events
        let collections = collection_events::table
            .filter(collection_events::specimen_id.eq(specimen_id))
            .load::<CollectionEvent>(&mut conn)
            .await?;

        for collection in collections.into_iter() {
            let entry = event_map.entry(collection.event_id);
            entry.or_default().push(EventDetails::Collection(collection.into()));
        }

        // get all sequencing events
        let sequencing = sequencing_events::table
            .inner_join(sequences::table)
            .inner_join(dna_extracts::table.on(dna_extracts::id.eq(sequences::dna_extract_id)))
            .inner_join(subsamples::table.on(subsamples::id.eq(dna_extracts::subsample_id)))
            .select(sequencing_events::all_columns)
            .filter(subsamples::specimen_id.eq(specimen_id))
            .load::<SequencingEvent>(&mut conn)
            .await?;

        let sequencing_ids: Vec<Uuid> = sequencing.iter().map(|s| s.id.clone()).collect();
        let runs = sequencing_run_events::table
            .filter(sequencing_run_events::sequencing_event_id.eq_any(sequencing_ids))
            .load::<SequencingRunEvent>(&mut conn)
            .await?;

        let mut sequence_map: HashMap<Uuid, Vec<SequencingRunEvent>> = HashMap::new();
        for run in runs {
            let entry = sequence_map.entry(run.sequencing_event_id);
            entry.or_default().push(run);
        }

        for sequencing in sequencing.into_iter() {
            let uuid = sequencing.id.clone();
            let event_id = sequencing.event_id.clone();

            let mut sequencing = specimen::SequencingEvent::from(sequencing);
            if let Some(runs) = sequence_map.remove(&uuid) {
                sequencing.runs = runs.into_iter().map(|r| specimen::SequencingRunEvent::from(r)).collect();
            }

            let entry = event_map.entry(event_id);
            entry.or_default().push(EventDetails::Sequencing(sequencing));
        }

        // add all sub events into their main event
        let events = events::table
            .filter(events::id.eq_any(event_map.keys()))
            .load::<Event>(&mut conn)
            .await?;

        let mut extended_events = Vec::new();
        for event in events {
            let uuid = event.id.clone();
            let mut ev = specimen::Event::from(event);

            if let Some(collections) = event_map.remove(&uuid) {
                ev.events = collections;
            }
            extended_events.push(ev);
        }

        Ok(extended_events)
    }
}

impl From<Event> for specimen::Event {
    fn from(value: Event) -> Self {
        Self {
            id: value.id.to_string(),
            field_number: value.field_number,
            event_date: value.event_date.map(|d| d.to_string()),
            habitat: value.habitat,
            sampling_protocol: value.sampling_protocol,
            sampling_size_value: value.sampling_size_value,
            sampling_size_unit: value.sampling_size_unit,
            sampling_effort: value.sampling_effort,
            field_notes: value.field_notes,
            event_remarks: value.event_remarks,
            events: Vec::new(),
        }
    }
}

impl From<CollectionEvent> for specimen::CollectionEvent {
    fn from(value: CollectionEvent) -> Self {
        Self {
            id: value.id.to_string(),
            catalog_number: value.catalog_number,
            record_number: value.record_number,
            individual_count: value.individual_count,
            organism_quantity: value.organism_quantity,
            organism_quantity_type: value.organism_quantity_type,
            sex: value.sex,
            life_stage: value.life_stage,
            reproductive_condition: value.reproductive_condition,
            behavior: value.behavior,
            establishment_means: value.establishment_means,
            degree_of_establishment: value.degree_of_establishment,
            pathway: value.pathway,
            occurrence_status: value.occurrence_status,
            preparation: value.preparation,
            other_catalog_numbers: value.other_catalog_numbers
        }
    }
}

impl From<SequencingEvent> for specimen::SequencingEvent {
    fn from(value: SequencingEvent) -> Self {
        Self {
            id: value.id.to_string(),
            target_gene: value.target_gene,
            dna_sequence: value.dna_sequence,
            runs: Vec::new(),
        }
    }
}

impl From<SequencingRunEvent> for specimen::SequencingRunEvent {
    fn from(value: SequencingRunEvent) -> Self {
        Self {
            id: value.id.to_string(),
            trace_id: value.trace_id,
            trace_name: value.trace_name,
            trace_link: value.trace_link,
            sequencing_date: value.sequencing_date.map(|d| d.to_string()),
            sequencing_center: value.sequencing_center,
            target_gene: value.target_gene,
            direction: value.direction,
            pcr_primer_name_forward: value.pcr_primer_name_forward,
            pcr_primer_name_reverse: value.pcr_primer_name_reverse,
            sequence_primer_forward_name: value.sequence_primer_forward_name,
            sequence_primer_reverse_name: value.sequence_primer_reverse_name,
        }
    }
}
