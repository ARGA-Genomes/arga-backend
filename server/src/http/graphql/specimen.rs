use async_graphql::*;
use tracing::instrument;

use crate::database::{models, Database};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum SpecimenBy {
    EntityId(String),
    RecordId(String),
    SequenceRecordId(String),
    SequenceAccession(String),
}

#[derive(MergedObject)]
pub struct Specimen(SpecimenDetails, SpecimenQuery);

impl Specimen {
    pub async fn new(db: &Database, by: &SpecimenBy) -> Result<Specimen, Error> {
        let specimen = match by {
            SpecimenBy::EntityId(id) => db.specimens.find_by_id(&id).await?,
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

    async fn organism(&self, ctx: &Context<'_>) -> Result<Organism, Error> {
        let state = ctx.data::<State>()?;
        let organism = state.database.specimens.organism(&self.specimen.entity_id).await?;
        Ok(organism.into())
    }

    async fn collections(&self, ctx: &Context<'_>) -> Result<Vec<CollectionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let collections = state.database.specimens.collection_events(specimen_id).await?;
        Ok(collections.into_iter().map(|r| r.into()).collect())
    }

    async fn accessions(&self, ctx: &Context<'_>) -> Result<Vec<AccessionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let accessions = state.database.specimens.accession_events(specimen_id).await?;
        Ok(accessions.into_iter().map(|r| r.into()).collect())
    }

    #[instrument(skip(self, ctx))]
    async fn events(&self, ctx: &Context<'_>) -> Result<SpecimenEvents, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let collections = state.database.specimens.collection_events(specimen_id).await?;
        let accessions = state.database.specimens.accession_events(specimen_id).await?;

        Ok(SpecimenEvents {
            collections: collections.into_iter().map(|r| r.into()).collect(),
            accessions: accessions.into_iter().map(|r| r.into()).collect(),
        })
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SpecimenDetails {
    pub entity_id: String,
    pub organism_id: String,
}

impl From<models::Specimen> for SpecimenDetails {
    fn from(value: models::Specimen) -> Self {
        Self {
            entity_id: value.entity_id,
            organism_id: value.organism_id,
        }
    }
}


#[derive(SimpleObject)]
pub struct SpecimenEvents {
    collections: Vec<CollectionEvent>,
    accessions: Vec<AccessionEvent>,
}


#[derive(Clone, Debug, SimpleObject)]
pub struct Organism {
    pub entity_id: String,
    pub organism_id: String,
    pub sex: Option<String>,
    pub genotypic_sex: Option<String>,
    pub phenotypic_sex: Option<String>,
    pub life_stage: Option<String>,
    pub reproductive_condition: Option<String>,
    pub behavior: Option<String>,
}

impl From<models::Organism> for Organism {
    fn from(value: models::Organism) -> Self {
        Organism {
            entity_id: value.entity_id,
            organism_id: value.organism_id,
            sex: value.sex,
            genotypic_sex: value.genotypic_sex,
            phenotypic_sex: value.phenotypic_sex,
            life_stage: value.life_stage,
            reproductive_condition: value.reproductive_condition,
            behavior: value.behavior,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CollectionEvent {
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

impl From<models::CollectionEvent> for CollectionEvent {
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
