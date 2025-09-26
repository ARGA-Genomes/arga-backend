use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::AccessionEvent;


#[derive(Clone)]
pub struct RegistrationProvider {
    pub pool: PgPool,
}

impl RegistrationProvider {
    pub async fn find_by_id(&self, registration_id: &str) -> Result<Option<AccessionEvent>, Error> {
        use schema::accession_events;
        let mut conn = self.pool.get().await?;

        let registration = accession_events::table
            .select(AccessionEvent::as_select())
            .filter(accession_events::entity_id.eq(registration_id))
            .get_result::<AccessionEvent>(&mut conn)
            .await
            .optional()?;

        Ok(registration)
    }
}
