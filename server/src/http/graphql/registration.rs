use async_graphql::*;

use super::common::{Publication, RegistrationDetails};
use crate::database::{Database, models};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum RegistrationBy {
    Id(String),
}

#[derive(MergedObject)]
pub struct Registration(RegistrationDetails, RegistrationQuery);

impl Registration {
    pub async fn new(db: &Database, by: &RegistrationBy) -> Result<Option<Registration>, Error> {
        let registration = match by {
            RegistrationBy::Id(id) => db.registrations.find_by_id(&id).await?,
        };

        match registration {
            None => Ok(None),
            Some(registration) => Ok(Some(Self::from_record(registration))),
        }
    }

    pub fn from_record(registration: models::AccessionEvent) -> Registration {
        let details = registration.clone().into();
        let query = RegistrationQuery { registration };
        Registration(details, query)
    }
}


struct RegistrationQuery {
    registration: models::AccessionEvent,
}

#[Object]
impl RegistrationQuery {
    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;
        Ok(None)
    }
}
