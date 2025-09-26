use async_graphql::*;

use super::common::{CollectionDetails, Publication};
use crate::database::{Database, models};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum CollectionBy {
    Id(String),
}

#[derive(MergedObject)]
pub struct Collection(CollectionDetails, CollectionQuery);

impl Collection {
    pub async fn new(db: &Database, by: &CollectionBy) -> Result<Option<Collection>, Error> {
        let collection = match by {
            CollectionBy::Id(id) => db.collections.find_by_id(&id).await?,
        };

        match collection {
            None => Ok(None),
            Some(collection) => Ok(Some(Self::from_record(collection))),
        }
    }

    pub fn from_record(collection: models::CollectionEvent) -> Collection {
        let details = collection.clone().into();
        let query = CollectionQuery { collection };
        Collection(details, query)
    }
}


struct CollectionQuery {
    collection: models::CollectionEvent,
}

#[Object]
impl CollectionQuery {
    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;
        Ok(None)
    }
}
