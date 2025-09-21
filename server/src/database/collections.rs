use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::CollectionEvent;


#[derive(Clone)]
pub struct CollectionProvider {
    pub pool: PgPool,
}

impl CollectionProvider {
    pub async fn find_by_id(&self, collection_id: &str) -> Result<Option<CollectionEvent>, Error> {
        use schema::collection_events;
        let mut conn = self.pool.get().await?;

        let collection = collection_events::table
            .select(CollectionEvent::as_select())
            .filter(collection_events::entity_id.eq(collection_id))
            .get_result::<CollectionEvent>(&mut conn)
            .await
            .optional()?;

        Ok(collection)
    }
}
