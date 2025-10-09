use chrono::{DateTime, NaiveDate, Utc};
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{schema, schema_gnl};


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::specimens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Specimen {
    pub entity_id: String,
    pub organism_id: String,
    pub name_id: Uuid,
    pub specimen_id: Option<String>,
}


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::organisms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Organism {
    pub entity_id: String,
    pub name_id: Uuid,
    pub organism_id: String,
    pub sex: Option<String>,
    pub genotypic_sex: Option<String>,
    pub phenotypic_sex: Option<String>,
    pub life_stage: Option<String>,
    pub reproductive_condition: Option<String>,
    pub behavior: Option<String>,
    pub publication_id: Option<String>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::tissues)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tissue {
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


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::collection_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CollectionEvent {
    pub entity_id: String,
    pub specimen_id: String,
    pub name_id: Uuid,
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

#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::accession_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccessionEvent {
    pub entity_id: String,
    pub specimen_id: String,
    pub name_id: Uuid,

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

#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::specimen_stats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SpecimenStats {
    pub entity_id: String,
    pub sequences: i64,
    pub whole_genomes: i64,
    pub loci: i64,
    pub other_genomic: i64,
    pub full_genomes: i64,
    pub partial_genomes: i64,
    pub complete_genomes: i64,
    pub assembly_chromosomes: i64,
    pub assembly_scaffolds: i64,
    pub assembly_contigs: i64,
}
