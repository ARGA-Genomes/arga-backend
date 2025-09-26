use async_graphql::*;

use super::common::{Publication, TissueDetails};
use crate::database::{Database, models};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum TissueBy {
    Id(String),
}

#[derive(MergedObject)]
pub struct Tissue(TissueDetails, TissueQuery);

impl Tissue {
    pub async fn new(db: &Database, by: &TissueBy) -> Result<Option<Tissue>, Error> {
        let tissue = match by {
            TissueBy::Id(id) => db.tissues.find_by_id(&id).await?,
        };

        match tissue {
            None => Ok(None),
            Some(tissue) => Ok(Some(Self::from_record(tissue))),
        }
    }

    pub fn from_record(tissue: models::Tissue) -> Tissue {
        let details = tissue.clone().into();
        let query = TissueQuery { tissue };
        Tissue(details, query)
    }
}


struct TissueQuery {
    tissue: models::Tissue,
}

#[Object]
impl TissueQuery {
    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;
        Ok(None)
    }
}
