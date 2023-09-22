use async_graphql::*;
use uuid::Uuid;

use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;


#[derive(OneofObject)]
pub enum SubsampleBy {
    Id(Uuid),
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
    async fn events(&self, ctx: &Context<'_>) -> Result<SubsampleEvents, Error> {
        let state = ctx.data::<State>().unwrap();
        let subsamples = state.database.subsamples.subsample_events(&self.subsample.id).await?;

        Ok(SubsampleEvents {
            subsamples: subsamples.into_iter().map(|r| r.into()).collect(),
        })
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SubsampleDetails {
    pub id: Uuid,
    pub specimen_id: Uuid,

    pub record_id: String,
    pub material_sample_id: Option<String>,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub type_status: Option<String>,
}

impl From<models::Subsample> for SubsampleDetails {
    fn from(value: models::Subsample) -> Self {
        Self {
            id: value.id,
            specimen_id: value.specimen_id,
            record_id: value.record_id,
            material_sample_id: value.material_sample_id,
            institution_name: value.institution_name,
            institution_code: value.institution_code,
            type_status: value.type_status,
        }
    }
}


#[derive(SimpleObject)]
pub struct SubsampleEvents {
    subsamples: Vec<SubsampleEvent>,
}


#[derive(Clone, Debug, SimpleObject)]
pub struct SubsampleEvent {
    pub id: Uuid,
    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub subsampled_by: Option<String>,
    pub preparation_type: Option<String>,
}

impl From<models::SubsampleEvent> for SubsampleEvent {
    fn from(value: models::SubsampleEvent) -> Self {
        Self {
            id: value.id,
            event_date: value.event_date,
            event_time: value.event_time,
            subsampled_by: value.subsampled_by,
            preparation_type: value.preparation_type,
        }
    }
}
