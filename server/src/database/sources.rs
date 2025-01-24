use arga_core::models::Species;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::extensions::Paginate;
use super::models::{Dataset, Source};
use super::{schema, schema_gnl, PageResult, PgPool};
use crate::database::extensions::filters::{with_filters, Filter};
use crate::database::Error;


const ALA_DATASET_ID: &str = "ARGA:TL:0001013";


#[derive(Clone)]
pub struct SourceProvider {
    pub pool: PgPool,
}

impl SourceProvider {
    pub async fn all_records(&self) -> Result<Vec<Source>, Error> {
        use schema::sources::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = sources.order_by(name).load::<Source>(&mut conn).await?;

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

    pub async fn species(
        &self,
        source: &Source,
        filters: &Vec<Filter>,
        page: i64,
        page_size: i64,
    ) -> PageResult<Species> {
        use schema::{datasets, name_attributes as attrs, taxon_names};
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let query = match with_filters(&filters) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let records = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::all_columns)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .order_by(species::scientific_name)
            .paginate(page)
            .per_page(page_size)
            .load::<(Species, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }
}
