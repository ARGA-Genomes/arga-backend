use async_graphql::*;

use super::common::{AnnotationDetails, Publication};
use crate::database::{Database, models};
use crate::http::Error;


#[derive(OneofObject)]
pub enum AnnotationBy {
    EntityId(String),
}

#[derive(MergedObject)]
pub struct Annotation(AnnotationDetails, AnnotationQuery);

impl Annotation {
    pub async fn new(db: &Database, by: &AnnotationBy) -> Result<Annotation, Error> {
        let annotation = match by {
            AnnotationBy::EntityId(id) => db.annotations.find_by_id(&id).await?,
        };
        Ok(annotation.into())
    }

    pub fn from_record(annotation: models::Annotation) -> Annotation {
        let details = annotation.clone().into();
        let query = AnnotationQuery { annotation };
        Annotation(details, query)
    }
}

impl From<models::Annotation> for Annotation {
    fn from(value: models::Annotation) -> Self {
        Self::from_record(value)
    }
}


struct AnnotationQuery {
    annotation: models::Annotation,
}

#[Object]
impl AnnotationQuery {
    async fn publication(&self, _ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        Ok(None)
    }
}
