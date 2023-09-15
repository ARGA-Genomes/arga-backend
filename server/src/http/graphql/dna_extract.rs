use async_graphql::*;
use uuid::Uuid;

use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;


#[derive(MergedObject)]
pub struct DnaExtract(DnaExtractDetails, DnaExtractQuery);

impl DnaExtract {
    pub async fn new(db: &Database, dna_extract_id: &Uuid) -> Result<DnaExtract, Error> {
        let dna_extract = db.dna_extracts.find_by_id(&dna_extract_id).await?;
        let details = dna_extract.clone().into();
        let query = DnaExtractQuery { dna_extract };
        Ok(DnaExtract(details, query))
    }
}


struct DnaExtractQuery {
    dna_extract: models::DnaExtract,
}

#[Object]
impl DnaExtractQuery {
    async fn events(&self, ctx: &Context<'_>) -> Result<DnaExtractEvents, Error> {
        let state = ctx.data::<State>().unwrap();
        let extracts = state.database.dna_extracts.dna_extraction_events(&self.dna_extract.id).await?;

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
    pub accession: String,
}

impl From<models::DnaExtract> for DnaExtractDetails {
    fn from(value: models::DnaExtract) -> Self {
        Self {
            id: value.id,
            subsample_id: value.subsample_id,
            accession: value.accession,
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
