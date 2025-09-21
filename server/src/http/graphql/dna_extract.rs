use async_graphql::*;

use super::common::{Agent, DnaExtractDetails, Publication};
use crate::database::{Database, models};
use crate::http::{Context as State, Error};


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
            Some(extract) => Ok(Some(Self::from_record(extract))),
        }
    }

    pub fn from_record(extract: models::DnaExtract) -> DnaExtract {
        let details = extract.clone().into();
        let query = DnaExtractQuery { extract };
        DnaExtract(details, query)
    }
}


struct DnaExtractQuery {
    extract: models::DnaExtract,
}

#[Object]
impl DnaExtractQuery {
    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;

        let publication = match &self.extract.publication_id {
            None => None,
            Some(publication_id) => Some(state.database.publications.find_by_id(publication_id).await?.into()),
        };

        Ok(publication)
    }

    async fn extracted_by(&self, ctx: &Context<'_>) -> Result<Option<Agent>, Error> {
        let state = ctx.data::<State>()?;

        let agent = match &self.extract.extracted_by {
            None => None,
            Some(agent_id) => Some(state.database.agents.find_by_id(agent_id).await?.into()),
        };

        Ok(agent)
    }
}
