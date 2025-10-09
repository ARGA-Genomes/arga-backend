use async_graphql::*;
use chrono::{DateTime, NaiveDate, Utc};

use crate::database::models;


#[derive(Clone, Debug, SimpleObject)]
pub struct OrganismDetails {
    pub entity_id: String,
    pub organism_id: String,
    pub sex: Option<String>,
    pub genotypic_sex: Option<String>,
    pub phenotypic_sex: Option<String>,
    pub life_stage: Option<String>,
    pub reproductive_condition: Option<String>,
    pub behavior: Option<String>,
    pub live_state: Option<String>,
    pub remarks: Option<String>,
    pub identified_by: Option<String>,
    pub identification_date: Option<NaiveDate>,
    pub disposition: Option<String>,
    pub first_observed_at: Option<NaiveDate>,
    pub last_known_alive_at: Option<NaiveDate>,
    pub biome: Option<String>,
    pub habitat: Option<String>,
    pub bioregion: Option<String>,
    pub ibra_imcra: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coordinate_system: Option<String>,
    pub location_source: Option<String>,
    pub holding: Option<String>,
    pub holding_id: Option<String>,
    pub holding_permit: Option<String>,
    pub record_created_at: Option<DateTime<Utc>>,
    pub record_updated_at: Option<DateTime<Utc>>,
}

impl From<models::Organism> for OrganismDetails {
    fn from(value: models::Organism) -> Self {
        OrganismDetails {
            entity_id: value.entity_id,
            organism_id: value.organism_id,
            sex: value.sex,
            genotypic_sex: value.genotypic_sex,
            phenotypic_sex: value.phenotypic_sex,
            life_stage: value.life_stage,
            reproductive_condition: value.reproductive_condition,
            behavior: value.behavior,
            live_state: value.live_state,
            remarks: value.remarks,
            identified_by: value.identified_by,
            identification_date: value.identification_date,
            disposition: value.disposition,
            first_observed_at: value.first_observed_at,
            last_known_alive_at: value.last_known_alive_at,
            biome: value.biome,
            habitat: value.habitat,
            bioregion: value.bioregion,
            ibra_imcra: value.ibra_imcra,
            latitude: value.latitude,
            longitude: value.longitude,
            coordinate_system: value.coordinate_system,
            location_source: value.location_source,
            holding: value.holding,
            holding_id: value.holding_id,
            holding_permit: value.holding_permit,
            record_created_at: value.record_created_at,
            record_updated_at: value.record_updated_at,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CollectionDetails {
    pub entity_id: String,
    pub specimen_id: String,
    pub organism_id: String,
    pub field_collecting_id: Option<String>,

    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub collected_by: Option<String>,
    pub collection_remarks: Option<String>,
    pub identified_by: Option<String>,
    pub identified_date: Option<chrono::NaiveDate>,
    pub identification_remarks: Option<String>,

    pub locality: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub state_province: Option<String>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub depth: Option<f64>,
    pub elevation_accuracy: Option<f64>,
    pub depth_accuracy: Option<f64>,
    pub location_source: Option<String>,

    pub preparation: Option<String>,
    pub environment_broad_scale: Option<String>,
    pub environment_local_scale: Option<String>,
    pub environment_medium: Option<String>,
    pub habitat: Option<String>,
    pub specific_host: Option<String>,
    pub individual_count: Option<String>,
    pub organism_quantity: Option<String>,
    pub organism_quantity_type: Option<String>,

    pub strain: Option<String>,
    pub isolate: Option<String>,
    pub field_notes: Option<String>,
}

impl From<models::CollectionEvent> for CollectionDetails {
    fn from(value: models::CollectionEvent) -> Self {
        Self {
            entity_id: value.entity_id,
            specimen_id: value.specimen_id,
            organism_id: value.organism_id,
            field_collecting_id: value.field_collecting_id,
            event_date: value.event_date,
            event_time: value.event_time,
            collected_by: value.collected_by,
            collection_remarks: value.collection_remarks,
            identified_by: value.identified_by,
            identified_date: value.identified_date,
            identification_remarks: value.identification_remarks,
            locality: value.locality,
            country: value.country,
            country_code: value.country_code,
            state_province: value.state_province,
            county: value.county,
            municipality: value.municipality,
            latitude: value.latitude,
            longitude: value.longitude,
            elevation: value.elevation,
            depth: value.depth,
            elevation_accuracy: value.elevation_accuracy,
            depth_accuracy: value.depth_accuracy,
            location_source: value.location_source,
            preparation: value.preparation,
            environment_broad_scale: value.environment_broad_scale,
            environment_local_scale: value.environment_local_scale,
            environment_medium: value.environment_medium,
            habitat: value.habitat,
            specific_host: value.specific_host,
            individual_count: value.individual_count,
            organism_quantity: value.organism_quantity,
            organism_quantity_type: value.organism_quantity_type,
            strain: value.strain,
            isolate: value.isolate,
            field_notes: value.field_notes,
        }
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct CollectionEvent {
    pub entity_id: String,
    pub specimen_id: String,
    pub organism_id: String,
    pub material_sample_id: Option<String>,
    pub field_collecting_id: Option<String>,

    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub collected_by: Option<String>,
    pub collection_remarks: Option<String>,
    pub identified_by: Option<String>,
    pub identified_date: Option<chrono::NaiveDate>,
    pub identification_remarks: Option<String>,

    pub locality: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub state_province: Option<String>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub depth: Option<f64>,
    pub elevation_accuracy: Option<f64>,
    pub depth_accuracy: Option<f64>,
    pub location_source: Option<String>,

    pub preparation: Option<String>,
    pub environment_broad_scale: Option<String>,
    pub environment_local_scale: Option<String>,
    pub environment_medium: Option<String>,
    pub habitat: Option<String>,
    pub specific_host: Option<String>,
    pub individual_count: Option<String>,
    pub organism_quantity: Option<String>,
    pub organism_quantity_type: Option<String>,

    pub strain: Option<String>,
    pub isolate: Option<String>,
    pub field_notes: Option<String>,
}

impl From<models::CollectionEvent> for CollectionEvent {
    fn from(value: models::CollectionEvent) -> Self {
        Self {
            entity_id: value.entity_id,
            specimen_id: value.specimen_id,
            organism_id: value.organism_id,
            material_sample_id: value.material_sample_id,
            field_collecting_id: value.field_collecting_id,
            event_date: value.event_date,
            event_time: value.event_time,
            collected_by: value.collected_by,
            collection_remarks: value.collection_remarks,
            identified_by: value.identified_by,
            identified_date: value.identified_date,
            identification_remarks: value.identification_remarks,
            locality: value.locality,
            country: value.country,
            country_code: value.country_code,
            state_province: value.state_province,
            county: value.county,
            municipality: value.municipality,
            latitude: value.latitude,
            longitude: value.longitude,
            elevation: value.elevation,
            depth: value.depth,
            elevation_accuracy: value.elevation_accuracy,
            depth_accuracy: value.depth_accuracy,
            location_source: value.location_source,
            preparation: value.preparation,
            environment_broad_scale: value.environment_broad_scale,
            environment_local_scale: value.environment_local_scale,
            environment_medium: value.environment_medium,
            habitat: value.habitat,
            specific_host: value.specific_host,
            individual_count: value.individual_count,
            organism_quantity: value.organism_quantity,
            organism_quantity_type: value.organism_quantity_type,
            strain: value.strain,
            isolate: value.isolate,
            field_notes: value.field_notes,
        }
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct AccessionEvent {
    pub entity_id: String,
    pub specimen_id: String,
    pub type_status: Option<String>,
    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub collection_repository_id: Option<String>,
    pub collection_repository_code: Option<String>,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub disposition: Option<String>,
    pub preparation: Option<String>,
    pub accessioned_by: Option<String>,
    pub prepared_by: Option<String>,
    pub identified_by: Option<String>,
    pub identified_date: Option<chrono::NaiveDate>,
    pub identification_remarks: Option<String>,
    pub other_catalog_numbers: Option<String>,
}

impl From<models::AccessionEvent> for AccessionEvent {
    fn from(value: models::AccessionEvent) -> Self {
        Self {
            entity_id: value.entity_id,
            specimen_id: value.specimen_id,
            type_status: value.type_status,
            event_date: value.event_date,
            event_time: value.event_time,
            institution_name: value.institution_name,
            institution_code: value.institution_code,
            collection_repository_id: value.collection_repository_id,
            collection_repository_code: value.collection_repository_code,
            disposition: value.disposition,
            preparation: value.preparation,
            accessioned_by: value.accessioned_by,
            prepared_by: value.prepared_by,
            identified_by: value.identified_by,
            identified_date: value.identified_date,
            identification_remarks: value.identification_remarks,
            other_catalog_numbers: value.other_catalog_numbers,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct RegistrationDetails {
    pub entity_id: String,
    pub specimen_id: String,
    pub type_status: Option<String>,
    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub collection_repository_id: Option<String>,
    pub collection_repository_code: Option<String>,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub disposition: Option<String>,
    pub preparation: Option<String>,
    pub accessioned_by: Option<String>,
    pub prepared_by: Option<String>,
    pub identified_by: Option<String>,
    pub identified_date: Option<chrono::NaiveDate>,
    pub identification_remarks: Option<String>,
    pub other_catalog_numbers: Option<String>,
}

impl From<models::AccessionEvent> for RegistrationDetails {
    fn from(value: models::AccessionEvent) -> Self {
        Self {
            entity_id: value.entity_id,
            specimen_id: value.specimen_id,
            type_status: value.type_status,
            event_date: value.event_date,
            event_time: value.event_time,
            institution_name: value.institution_name,
            institution_code: value.institution_code,
            collection_repository_id: value.collection_repository_id,
            collection_repository_code: value.collection_repository_code,
            disposition: value.disposition,
            preparation: value.preparation,
            accessioned_by: value.accessioned_by,
            prepared_by: value.prepared_by,
            identified_by: value.identified_by,
            identified_date: value.identified_date,
            identification_remarks: value.identification_remarks,
            other_catalog_numbers: value.other_catalog_numbers,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct TissueDetails {
    pub entity_id: String,
    pub specimen_id: String,
    pub material_sample_id: String,
    pub tissue_id: String,
    pub identification_verified: Option<bool>,
    pub reference_material: Option<bool>,
    pub custodian: Option<String>,
    pub institution: Option<String>,
    pub institution_code: Option<String>,
    pub sampling_protocol: Option<String>,
    pub tissue_type: Option<String>,
    pub disposition: Option<String>,
    pub fixation: Option<String>,
    pub storage: Option<String>,
}

impl From<models::Tissue> for TissueDetails {
    fn from(value: models::Tissue) -> Self {
        Self {
            entity_id: value.entity_id,
            specimen_id: value.specimen_id,
            material_sample_id: value.material_sample_id,
            tissue_id: value.tissue_id,
            identification_verified: value.identification_verified,
            reference_material: value.reference_material,
            custodian: value.custodian,
            institution: value.institution,
            institution_code: value.institution_code,
            sampling_protocol: value.sampling_protocol,
            tissue_type: value.tissue_type,
            disposition: value.disposition,
            fixation: value.fixation,
            storage: value.storage,
        }
    }
}
