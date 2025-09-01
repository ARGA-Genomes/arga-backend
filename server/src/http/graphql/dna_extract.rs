use async_graphql::*;
use tracing::{instrument, info};
use uuid::Uuid;

use crate::database::{models, Database};
use crate::http::{Context as State, Error};


#[derive(OneofObject, Debug)]
pub enum DnaExtractBy {
    Id(Uuid),
    RecordId(String),
    SpecimenRecordId(String),
}

#[derive(MergedObject)]
pub struct DnaExtract(DnaExtractDetails, DnaExtractQuery);

impl DnaExtract {
    #[instrument(skip(db), fields(dna_extract_by = ?by))]
    pub async fn new(db: &Database, by: &DnaExtractBy) -> Result<Option<DnaExtract>, Error> {
        info!("Creating new DNA extract query");
        let dna_extract = match by {
            DnaExtractBy::Id(id) => db.dna_extracts.find_by_id(&id).await?,
            DnaExtractBy::RecordId(id) => db.dna_extracts.find_by_record_id(&id).await?,
            DnaExtractBy::SpecimenRecordId(id) => db.dna_extracts.find_by_specimen_record_id(&id).await?,
        };

        match dna_extract {
            None => Ok(None),
            Some(dna_extract) => {
                let details = dna_extract.clone().into();
                let query = DnaExtractQuery { dna_extract };
                Ok(Some(DnaExtract(details, query)))
            }
        }
    }
}


struct DnaExtractQuery {
    dna_extract: models::DnaExtract,
}

#[Object]
impl DnaExtractQuery {
    #[instrument(skip(self, ctx))]
    async fn events(&self, ctx: &Context<'_>) -> Result<DnaExtractEvents, Error> {
        info!("Fetching DNA extract events");
        let state = ctx.data::<State>()?;
        let extracts = state
            .database
            .dna_extracts
            .dna_extraction_events(&self.dna_extract.id)
            .await?;

        Ok(DnaExtractEvents {
            dna_extracts: extracts.into_iter().map(|r| r.into()).collect(),
        })
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct DnaExtractDetails {
    pub id: Uuid,
    pub subsample_id: Uuid,
    pub record_id: String,
}

impl From<models::DnaExtract> for DnaExtractDetails {
    fn from(value: models::DnaExtract) -> Self {
        Self {
            id: value.id,
            subsample_id: value.subsample_id,
            record_id: value.record_id,
        }
    }
}


#[derive(SimpleObject)]
pub struct DnaExtractEvents {
    dna_extracts: Vec<DnaExtractionEvent>,
}


#[derive(Clone, Debug, SimpleObject)]
pub struct DnaExtractionEvent {
    pub id: Uuid,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub extracted_by: Option<String>,

    pub preservation_type: Option<String>,
    pub preparation_type: Option<String>,
    pub extraction_method: Option<String>,
    pub measurement_method: Option<String>,
    pub concentration_method: Option<String>,
    pub quality: Option<String>,

    pub concentration: Option<f64>,
    pub absorbance_260_230: Option<f64>,
    pub absorbance_260_280: Option<f64>,
}

impl From<models::DnaExtractionEvent> for DnaExtractionEvent {
    fn from(value: models::DnaExtractionEvent) -> Self {
        Self {
            id: value.id,
            event_date: value.event_date,
            event_time: value.event_time,
            extracted_by: value.extracted_by,
            preservation_type: value.preservation_type,
            preparation_type: value.preparation_type,
            extraction_method: value.extraction_method,
            measurement_method: value.measurement_method,
            concentration_method: value.concentration_method,
            quality: value.quality,
            concentration: value.concentration,
            absorbance_260_230: value.absorbance_260_230,
            absorbance_260_280: value.absorbance_260_280,
        }
    }
}
