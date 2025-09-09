use async_graphql::*;

use crate::database::{Database, models};
use crate::http::Error;


#[derive(OneofObject)]
pub enum SubsampleBy {
    Id(String),
    RecordId(String),
    SpecimenRecordId(String),
}

#[derive(MergedObject)]
pub struct Subsample(SubsampleDetails, SubsampleQuery);

impl Subsample {
    pub async fn new(db: &Database, by: &SubsampleBy) -> Result<Option<Subsample>, Error> {
        let subsample = match by {
            SubsampleBy::Id(id) => db.subsamples.find_by_id(&id).await?,
            SubsampleBy::RecordId(id) => db.subsamples.find_by_record_id(&id).await?,
            SubsampleBy::SpecimenRecordId(id) => db.subsamples.find_by_specimen_record_id(&id).await?,
        };

        match subsample {
            None => Ok(None),
            Some(subsample) => {
                let details = subsample.clone().into();
                let query = SubsampleQuery { subsample };
                Ok(Some(Subsample(details, query)))
            }
        }
    }
}


struct SubsampleQuery {
    subsample: models::Subsample,
}

#[Object]
impl SubsampleQuery {
    async fn publication(&self, ctx: &Context<'_>) -> Result<String, Error> {
        Ok("".to_string())
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SubsampleDetails {
    pub entity_id: String,
    pub subsample_id: String,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub sample_type: Option<String>,
    pub name: Option<String>,
    pub custodian: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub culture_method: Option<String>,
    pub culture_media: Option<String>,
    pub weight_or_volume: Option<String>,
    pub preservation_method: Option<String>,
    pub preservation_temperature: Option<String>,
    pub preservation_duration: Option<String>,
    pub quality: Option<String>,
    pub cell_type: Option<String>,
    pub cell_line: Option<String>,
    pub clone_name: Option<String>,
    pub lab_host: Option<String>,
    pub sample_processing: Option<String>,
    pub sample_pooling: Option<String>,
}

impl From<models::Subsample> for SubsampleDetails {
    fn from(value: models::Subsample) -> Self {
        Self {
            entity_id: value.entity_id,
            subsample_id: value.subsample_id,
            institution_name: value.institution_name,
            institution_code: value.institution_code,
            event_date: value.event_date,
            event_time: value.event_time,
            sample_type: value.sample_type,
            name: value.name,
            custodian: value.custodian,
            description: value.description,
            notes: value.notes,
            culture_method: value.culture_method,
            culture_media: value.culture_media,
            weight_or_volume: value.weight_or_volume,
            preservation_method: value.preservation_method,
            preservation_temperature: value.preservation_temperature,
            preservation_duration: value.preservation_duration,
            quality: value.quality,
            cell_type: value.cell_type,
            cell_line: value.cell_line,
            clone_name: value.clone_name,
            lab_host: value.lab_host,
            sample_processing: value.sample_processing,
            sample_pooling: value.sample_pooling,
        }
    }
}
