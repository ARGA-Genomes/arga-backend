use async_graphql::*;

use super::common::DnaExtractDetails;
use crate::database::{Database, models};
use crate::http::Error;


#[derive(OneofObject)]
pub enum DnaExtractBy {
    Id(String),
    RecordId(String),
    SpecimenRecordId(String),
}

#[derive(MergedObject)]
pub struct DnaExtract(DnaExtractDetails, DnaExtractQuery);

impl DnaExtract {
    pub async fn new(db: &Database, by: &DnaExtractBy) -> Result<Option<DnaExtract>, Error> {
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
    async fn publication(&self, ctx: &Context<'_>) -> Result<String, Error> {
        Ok("".to_string())
    }
}
