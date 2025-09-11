use async_graphql::*;

use super::common::SubsampleDetails;
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
