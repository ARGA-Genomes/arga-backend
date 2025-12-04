use async_graphql::*;

use super::common::{DepositionDetails, Publication};
use crate::database::{Database, models};
use crate::http::Error;


#[derive(OneofObject)]
pub enum DepositionBy {
    EntityId(String),
}

#[derive(MergedObject)]
pub struct Deposition(DepositionDetails, DepositionQuery);

impl Deposition {
    pub async fn new(db: &Database, by: &DepositionBy) -> Result<Deposition, Error> {
        let deposition = match by {
            DepositionBy::EntityId(id) => db.depositions.find_by_id(&id).await?,
        };
        Ok(deposition.into())
    }

    pub fn from_record(deposition: models::Deposition) -> Deposition {
        let details = deposition.clone().into();
        let query = DepositionQuery { deposition };
        Deposition(details, query)
    }
}

impl From<models::Deposition> for Deposition {
    fn from(value: models::Deposition) -> Self {
        Self::from_record(value)
    }
}


struct DepositionQuery {
    deposition: models::Deposition,
}

#[Object]
impl DepositionQuery {
    async fn publication(&self, _ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        Ok(None)
    }
}
