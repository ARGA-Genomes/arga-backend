use async_graphql::*;

use super::common::{LibraryDetails, NameDetails, Publication};
use crate::database::{Database, models};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum LibraryBy {
    EntityId(String),
}

#[derive(MergedObject)]
pub struct Library(LibraryDetails, LibraryQuery);

impl Library {
    pub async fn new(db: &Database, by: &LibraryBy) -> Result<Library, Error> {
        let library = match by {
            LibraryBy::EntityId(id) => db.libraries.find_by_id(&id).await?,
        };
        Ok(library.into())
    }

    pub fn from_record(library: models::Library) -> Library {
        let details = library.clone().into();
        let query = LibraryQuery { library };
        Library(details, query)
    }
}

impl From<models::Library> for Library {
    fn from(value: models::Library) -> Self {
        Self::from_record(value)
    }
}


struct LibraryQuery {
    library: models::Library,
}

#[Object]
impl LibraryQuery {
    async fn name(&self, ctx: &Context<'_>) -> Result<NameDetails, Error> {
        let state = ctx.data::<State>()?;
        let name = state
            .database
            .names
            .find_by_entity_id(&self.library.species_name_id)
            .await?;
        Ok(name.into())
    }

    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;

        let publication = match &self.library.publication_id {
            None => None,
            Some(publication_id) => Some(state.database.publications.find_by_id(publication_id).await?.into()),
        };

        Ok(publication)
    }
}
