use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::Publication;


#[derive(Clone)]
pub struct PublicationProvider {
    pub pool: PgPool,
}

impl PublicationProvider {
    pub async fn find_by_id(&self, entity_id: &str) -> Result<Publication, Error> {
        use schema::publications;
        let mut conn = self.pool.get().await?;

        let publication = publications::table
            .filter(publications::entity_id.eq(entity_id))
            .select(Publication::as_select())
            .get_result::<Publication>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = publication {
            return Err(Error::NotFound(entity_id.to_string()));
        }

        Ok(publication?)
    }
}
