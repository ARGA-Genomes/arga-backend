use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use arga_core::models;
use crate::http::Error;
use crate::http::Context as State;

use super::dataset::DatasetDetails;



#[derive(MergedObject)]
pub struct Source(SourceDetails, SourceQuery);

impl Source {
    pub fn new(source: models::Source) -> Source {
        let details = source.clone().into();
        let query = SourceQuery { source };
        Source(details, query)
    }
}


pub struct SourceQuery {
    source: models::Source,
}

#[Object]
impl SourceQuery {
    async fn datasets(&self, ctx: &Context<'_>) -> Result<Vec<DatasetDetails>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.database.sources.datasets(&self.source).await?;
        let datasets = records.into_iter().map(|dataset| dataset.into()).collect();
        Ok(datasets)
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SourceDetails {
    pub id: Uuid,
    pub name: String,
    pub author: String,
    pub rights_holder: String,
    pub access_rights: String,
    pub license: String,
}

impl From<models::Source> for SourceDetails {
    fn from(value: models::Source) -> Self {
        Self {
            id: value.id,
            name: value.name,
            author: value.author,
            rights_holder: value.rights_holder,
            access_rights: value.access_rights,
            license: value.license,
        }
    }
}
