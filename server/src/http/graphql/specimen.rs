use async_graphql::*;
use tracing::instrument;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;
use crate::database::schema;
use crate::index::names::GetNames;
use crate::index::specimen::{SpecimenDetails, Event};
use crate::index::specimen::GetSpecimenEvents;


#[derive(MergedObject)]
pub struct Specimen(SpecimenDetails, SpecimenQuery);

impl Specimen {
    pub async fn new(db: &Database, specimen_id: &str) -> Result<Specimen, Error> {
        let query = SpecimenQuery::new(db, specimen_id).await?;
        Ok(Specimen(query.specimen.clone().into(), query))
    }
}


struct SpecimenQuery {
    specimen: models::Specimen,
}

#[Object]
impl SpecimenQuery {
    #[graphql(skip)]
    pub async fn new(db: &Database, specimen_id: &str) -> Result<SpecimenQuery, Error> {
        use schema::specimens;
        let mut conn = db.pool.get().await?;
        let specimen_id = uuid::Uuid::parse_str(specimen_id).unwrap_or_default();

        let specimen = specimens::table
            .filter(specimens::id.eq(&specimen_id))
            .get_result::<models::Specimen>(&mut conn)
            .await?;

        Ok(SpecimenQuery {
            specimen,
        })
    }

    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
        let state = ctx.data::<State>().unwrap();
        let name = state.database.find_by_name_id(&self.specimen.name_id).await?;
        Ok(name.canonical_name)
    }

    #[instrument(skip(self))]
    async fn details(&self) -> SpecimenDetails {
        self.specimen.clone().into()
    }

    #[instrument(skip(self, ctx))]
    async fn events(&self, ctx: &Context<'_>) -> Result<Vec<Event>, Error> {
        let state = ctx.data::<State>().unwrap();
        let events = state.database.get_specimen_events(&self.specimen.id).await?;
        Ok(events)
    }
}
