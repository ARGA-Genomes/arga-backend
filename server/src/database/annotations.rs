use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::Annotation;


#[derive(Clone)]
pub struct AnnotationProvider {
    pub pool: PgPool,
}

impl AnnotationProvider {
    pub async fn find_by_id(&self, entity_id: &str) -> Result<Annotation, Error> {
        use schema::annotations;
        let mut conn = self.pool.get().await?;

        let annotation = annotations::table
            .filter(annotations::entity_id.eq(entity_id))
            .select(Annotation::as_select())
            .get_result::<Annotation>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = annotation {
            return Err(Error::NotFound(entity_id.to_string()));
        }

        Ok(annotation?)
    }

    pub async fn find_by_assembly_id(&self, entity_id: &str) -> Result<Vec<Annotation>, Error> {
        use schema::annotations;
        let mut conn = self.pool.get().await?;

        let records = annotations::table
            .filter(annotations::assembly_id.eq(entity_id))
            .select(Annotation::as_select())
            .load::<Annotation>(&mut conn)
            .await?;

        Ok(records)
    }
}
