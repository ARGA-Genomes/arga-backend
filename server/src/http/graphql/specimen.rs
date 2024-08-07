use async_graphql::*;
use tracing::instrument;
use uuid::Uuid;

use crate::database::{models, Database};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum SpecimenBy {
    Id(Uuid),
    RecordId(String),
    SequenceRecordId(String),
    SequenceAccession(String),
}

#[derive(MergedObject)]
pub struct Specimen(SpecimenDetails, SpecimenQuery);

impl Specimen {
    pub async fn new(db: &Database, by: &SpecimenBy) -> Result<Specimen, Error> {
        let specimen = match by {
            SpecimenBy::Id(id) => db.specimens.find_by_id(&id).await?,
            SpecimenBy::RecordId(id) => db.specimens.find_by_record_id(&id).await?,
            SpecimenBy::SequenceRecordId(id) => db.specimens.find_by_sequence_record_id(&id).await?,
            SpecimenBy::SequenceAccession(id) => db.specimens.find_by_sequence_accession(&id).await?,
        };
        let details = specimen.clone().into();
        let query = SpecimenQuery { specimen };
        Ok(Specimen(details, query))
    }
}


struct SpecimenQuery {
    specimen: models::Specimen,
}

#[Object]
impl SpecimenQuery {
    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let name = state.database.names.find_by_name_id(&self.specimen.name_id).await?;
        Ok(name.canonical_name)
    }

    #[instrument(skip(self, ctx))]
    async fn events(&self, ctx: &Context<'_>) -> Result<SpecimenEvents, Error> {
        let state = ctx.data::<State>()?;
        let collections = state.database.specimens.collection_events(&self.specimen.id).await?;
        let accessions = state.database.specimens.accession_events(&self.specimen.id).await?;

        Ok(SpecimenEvents {
            collections: collections.into_iter().map(|r| r.into()).collect(),
            accessions: accessions.into_iter().map(|r| r.into()).collect(),
        })
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SpecimenDetails {
    pub id: Uuid,
    pub entity_id: Option<String>,

    pub record_id: String,
    pub material_sample_id: Option<String>,
    pub organism_id: Option<String>,

    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub collection_code: Option<String>,
    pub recorded_by: Option<String>,
    pub identified_by: Option<String>,
    pub identified_date: Option<String>,

    pub type_status: Option<String>,
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

    pub details: Option<String>,
    pub remarks: Option<String>,
    pub identification_remarks: Option<String>,
}

impl From<models::Specimen> for SpecimenDetails {
    fn from(value: models::Specimen) -> Self {
        Self {
            id: value.id,
            entity_id: value.entity_id,
            record_id: value.record_id,
            material_sample_id: value.material_sample_id,
            organism_id: value.organism_id,
            institution_name: value.institution_name,
            institution_code: value.institution_code,
            collection_code: value.collection_code,
            recorded_by: value.recorded_by,
            identified_by: value.identified_by,
            identified_date: value.identified_date,
            type_status: value.type_status,
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
            details: value.details,
            remarks: value.remarks,
            identification_remarks: value.identification_remarks,
        }
    }
}


#[derive(SimpleObject)]
pub struct SpecimenEvents {
    collections: Vec<CollectionEvent>,
    accessions: Vec<AccessionEvent>,
}


#[derive(Clone, Debug, SimpleObject)]
pub struct CollectionEvent {
    pub id: Uuid,
    pub entity_id: Option<String>,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub collected_by: Option<String>,

    pub field_number: Option<String>,
    pub catalog_number: Option<String>,
    pub record_number: Option<String>,
    pub individual_count: Option<String>,
    pub organism_quantity: Option<String>,
    pub organism_quantity_type: Option<String>,
    pub sex: Option<String>,
    pub genotypic_sex: Option<String>,
    pub phenotypic_sex: Option<String>,
    pub life_stage: Option<String>,
    pub reproductive_condition: Option<String>,
    pub behavior: Option<String>,
    pub establishment_means: Option<String>,
    pub degree_of_establishment: Option<String>,
    pub pathway: Option<String>,
    pub occurrence_status: Option<String>,
    pub preparation: Option<String>,
    pub other_catalog_numbers: Option<String>,

    pub env_broad_scale: Option<String>,
    pub env_local_scale: Option<String>,
    pub env_medium: Option<String>,
    pub habitat: Option<String>,
    pub ref_biomaterial: Option<String>,
    pub source_mat_id: Option<String>,
    pub specific_host: Option<String>,
    pub strain: Option<String>,
    pub isolate: Option<String>,

    pub field_notes: Option<String>,
    pub remarks: Option<String>,
}

impl From<models::CollectionEvent> for CollectionEvent {
    fn from(value: models::CollectionEvent) -> Self {
        Self {
            id: value.id,
            entity_id: value.entity_id,
            event_date: value.event_date,
            event_time: value.event_time,
            collected_by: value.collected_by,
            field_number: value.field_number,
            catalog_number: value.catalog_number,
            record_number: value.record_number,
            individual_count: value.individual_count,
            organism_quantity: value.organism_quantity,
            organism_quantity_type: value.organism_quantity_type,
            sex: value.sex,
            genotypic_sex: value.genotypic_sex,
            phenotypic_sex: value.phenotypic_sex,
            life_stage: value.life_stage,
            reproductive_condition: value.reproductive_condition,
            behavior: value.behavior,
            establishment_means: value.establishment_means,
            degree_of_establishment: value.degree_of_establishment,
            pathway: value.pathway,
            occurrence_status: value.occurrence_status,
            preparation: value.preparation,
            other_catalog_numbers: value.other_catalog_numbers,
            env_broad_scale: value.env_broad_scale,
            env_local_scale: value.env_local_scale,
            env_medium: value.env_medium,
            habitat: value.habitat,
            ref_biomaterial: value.ref_biomaterial,
            source_mat_id: value.source_mat_id,
            specific_host: value.specific_host,
            strain: value.strain,
            isolate: value.isolate,
            field_notes: value.field_notes,
            remarks: value.remarks,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct AccessionEvent {
    pub id: Uuid,
    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub accession: String,
    pub accessioned_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub type_status: Option<String>,
}

impl From<models::AccessionEvent> for AccessionEvent {
    fn from(value: models::AccessionEvent) -> Self {
        Self {
            id: value.id,
            event_date: value.event_date,
            event_time: value.event_time,
            accession: value.accession,
            accessioned_by: value.accessioned_by,
            material_sample_id: value.material_sample_id,
            institution_name: value.institution_name,
            institution_code: value.institution_code,
            type_status: value.type_status,
        }
    }
}
