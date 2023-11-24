use arga_core::models::Taxon;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::Error;

use super::extensions::Paginate;
use super::{schema, PgPool, PageResult};
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

    pub async fn species(&self, source: &Source, page: i64, page_size: i64) -> PageResult<Taxon> {
        use schema::{datasets, taxa, names, name_attributes as attrs};
        let mut conn = self.pool.get().await?;

        let with_source = names::table
            .inner_join(attrs::table)
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .filter(datasets::source_id.eq(source.id))
            .select(names::id)
            .group_by(names::id)
            .into_boxed();

        let species = taxa::table
            .filter(taxa::name_id.eq_any(with_source))
            .order_by(taxa::scientific_name)
            .paginate(page)
            .per_page(page_size)
            .load::<(Taxon, i64)>(&mut conn)
            .await?;

        Ok(species.into())
    }
}
