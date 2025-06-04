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
    specimen: models::SpecimenOld,
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

impl From<models::SpecimenOld> for SpecimenDetails {
    fn from(value: models::SpecimenOld) -> Self {
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
    pub entity_id: String,

    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub collected_by: Option<String>,
    pub collection_remarks: Option<String>,
}

impl From<models::CollectionEvent> for CollectionEvent {
    fn from(value: models::CollectionEvent) -> Self {
        Self {
            entity_id: value.entity_id,
            event_date: value.event_date,
            event_time: value.event_time,
            collected_by: value.collected_by,
            collection_remarks: value.collection_remarks,
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
