use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Specimen, Organism, CollectionEvent, SequencingEvent, SequencingRunEvent};
use crate::index::specimen::{self, GetSpecimen};
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
