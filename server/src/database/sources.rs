use arga_core::models::Species;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::Error;
use crate::database::extensions::filters::{with_filters, Filter};

use super::extensions::Paginate;
use super::{schema, schema_gnl, PgPool, PageResult};
use super::models::{Source, Dataset};


#[derive(Clone)]
pub struct SourceProvider {
    pub pool: PgPool,
}

impl SourceProvider {
    pub async fn all_records(&self) -> Result<Vec<Source>, Error> {
        use schema::sources::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = sources
            .order_by(name)
            .load::<Source>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Source, Error> {
        use schema::sources;
        let mut conn = self.pool.get().await?;

        let source = sources::table
            .filter(sources::id.eq(id))
            .get_result::<Source>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = source {
            return Err(Error::NotFound(id.to_string()));
        }

        Ok(source?)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Source, Error> {
        use schema::sources;
        let mut conn = self.pool.get().await?;

        let source = sources::table
            .filter(sources::name.eq(name))
            .get_result::<Source>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = source {
            return Err(Error::NotFound(name.to_string()));
        }

        Ok(source?)
    }

    pub async fn datasets(&self, source: &Source) -> Result<Vec<Dataset>, Error> {
        use schema::datasets;
        let mut conn = self.pool.get().await?;

        let records = datasets::table
            .filter(datasets::source_id.eq(source.id))
            .order_by(datasets::name)
            .load::<Dataset>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn species(&self, source: &Source, filters: &Vec<Filter>, page: i64, page_size: i64) -> PageResult<Species> {
        use schema::{datasets, name_attributes as attrs};
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let records = species::table
            .inner_join(attrs::table.on(attrs::name_id.eq(species::id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .select(species::all_columns)
            .filter(datasets::source_id.eq(source.id))
            // FIXME: make filterable
            // .with_filters(&filters)
            .order_by(species::scientific_name)
            .paginate(page)
            .per_page(page_size)
            .load::<(Species, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }
}
