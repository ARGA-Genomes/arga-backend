use async_graphql::{SimpleObject, Union};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// A specimen of a specific species.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SpecimenDetails {
    pub id: String,
    pub type_status: Option<String>,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub collection_code: Option<String>,
    pub material_sample_id: Option<String>,
    pub recorded_by: Option<String>,
    pub organism_id: Option<String>,
    pub locality: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub details: Option<String>,
    pub remarks: Option<String>,
}

#[async_trait]
pub trait GetSpecimen {
    type Error;
    async fn get_specimen(&self, specimen_id: &Uuid) -> Result<SpecimenDetails, Self::Error>;
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Organism {
    pub id: String,
    pub organism_id: Option<String>,
    pub organism_name: Option<String>,
    pub organism_scope: Option<String>,
    pub associated_organisms: Option<String>,
    pub previous_identifications: Option<String>,
    pub remarks: Option<String>,
}

#[async_trait]
pub trait GetSpecimenOrganism {
    type Error;
    async fn get_specimen_organism(&self, specimen_id: &Uuid) -> Result<Organism, Self::Error>;
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Event {
    pub id: String,

    pub field_number: Option<String>,
    pub event_date: Option<String>,

    pub habitat: Option<String>,
    pub sampling_protocol: Option<String>,
    pub sampling_size_value: Option<String>,
    pub sampling_size_unit: Option<String>,
    pub sampling_effort: Option<String>,
    pub field_notes: Option<String>,
    pub event_remarks: Option<String>,

    pub events: Vec<EventDetails>,
}

#[derive(Debug, Clone, Union, Serialize, Deserialize)]
pub enum EventDetails {
    Collection(CollectionEvent),
    Sequencing(SequencingEvent),
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct CollectionEvent {
    pub id: String,
    pub catalog_number: Option<String>,
    pub record_number: Option<String>,
    pub individual_count: Option<String>,
    pub organism_quantity: Option<String>,
    pub organism_quantity_type: Option<String>,
    pub sex: Option<String>,
    pub life_stage: Option<String>,
    pub reproductive_condition: Option<String>,
    pub behavior: Option<String>,
    pub establishment_means: Option<String>,
    pub degree_of_establishment: Option<String>,
    pub pathway: Option<String>,
    pub occurrence_status: Option<String>,
    pub preparation: Option<String>,
    pub other_catalog_numbers: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SequencingEvent {
    pub id: String,
    pub target_gene: Option<String>,
    pub dna_sequence: Option<String>,
    pub runs: Vec<SequencingRunEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SequencingRunEvent {
    pub id: String,
    pub trace_id: Option<String>,
    pub trace_name: Option<String>,
    pub trace_link: Option<String>,
    pub sequencing_date: Option<String>,
    pub sequencing_center: Option<String>,
    pub target_gene: Option<String>,
    pub direction: Option<String>,
    pub pcr_primer_name_forward: Option<String>,
    pub pcr_primer_name_reverse: Option<String>,
    pub sequence_primer_forward_name: Option<String>,
    pub sequence_primer_reverse_name: Option<String>,
}


#[async_trait]
pub trait GetSpecimenEvents {
    type Error;

    async fn get_specimen_events(&self, specimen_id: &Uuid) -> Result<Vec<Event>, Self::Error>;
}
