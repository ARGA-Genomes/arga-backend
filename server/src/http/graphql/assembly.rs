use async_graphql::*;

use super::common::{AssemblyDetails, NameDetails, Publication};
use super::library::Library;
use super::specimen::Specimen;
use crate::database::{Database, models};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum AssemblyBy {
    EntityId(String),
}

#[derive(MergedObject)]
pub struct Assembly(AssemblyDetails, AssemblyQuery);

impl Assembly {
    pub async fn new(db: &Database, by: &AssemblyBy) -> Result<Assembly, Error> {
        let assembly = match by {
            AssemblyBy::EntityId(id) => db.assemblies.find_by_id(&id).await?,
        };
        let details = assembly.clone().into();
        let query = AssemblyQuery { assembly };
        Ok(Assembly(details, query))
    }
}


struct AssemblyQuery {
    assembly: models::Assembly,
}

#[Object]
impl AssemblyQuery {
    async fn name(&self, ctx: &Context<'_>) -> Result<NameDetails, Error> {
        let state = ctx.data::<State>()?;
        let name = state
            .database
            .names
            .find_by_entity_id(&self.assembly.species_name_id)
            .await?;
        Ok(name.into())
    }

    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;

        let publication = match &self.assembly.publication_id {
            None => None,
            Some(publication_id) => Some(state.database.publications.find_by_id(publication_id).await?.into()),
        };

        Ok(publication)
    }

    async fn libraries(&self, ctx: &Context<'_>) -> Result<Vec<Library>, Error> {
        let state = ctx.data::<State>()?;
        let records = state
            .database
            .libraries
            .find_by_assembly_id(&self.assembly.entity_id)
            .await?;

        Ok(records.into_iter().map(Library::from).collect())
    }

    async fn specimens(&self, ctx: &Context<'_>) -> Result<Vec<Specimen>, Error> {
        let state = ctx.data::<State>()?;
        let records = state.database.assemblies.specimens(&self.assembly.entity_id).await?;

        Ok(records.into_iter().map(Specimen::from).collect())
    }
}
