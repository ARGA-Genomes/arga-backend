use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::extensions::Paginate;
use super::models::{Dataset, Taxon};
use super::{PageResult, PgPool, schema};
use crate::database::Error;
use crate::database::sources::ALA_DATASET_ID;


#[derive(Clone)]
pub struct DatasetProvider {
    pub pool: PgPool,
}

impl DatasetProvider {
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Dataset, Error> {
        use schema::datasets;
        let mut conn = self.pool.get().await?;

        let dataset = datasets::table
            .filter(datasets::id.eq(id))
            .get_result::<Dataset>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = dataset {
            return Err(Error::NotFound(id.to_string()));
        }

        Ok(dataset?)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Dataset, Error> {
        use schema::datasets;
        let mut conn = self.pool.get().await?;

        let dataset = datasets::table
            .filter(datasets::name.eq(name))
            .get_result::<Dataset>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = dataset {
            return Err(Error::NotFound(name.to_string()));
        }

        Ok(dataset?)
    }

    pub async fn species(&self, dataset: &Dataset, page: i64) -> PageResult<Taxon> {
        use schema::{datasets, name_attributes, names, taxa, taxon_names};
        let mut conn = self.pool.get().await?;

        let species = name_attributes::table
            .inner_join(names::table)
            .inner_join(taxon_names::table.on(taxon_names::name_id.eq(names::id)))
            .inner_join(taxa::table.on(taxa::id.eq(taxon_names::taxon_id)))
            .inner_join(datasets::table.on(datasets::id.eq(taxa::dataset_id)))
            .filter(name_attributes::dataset_id.eq(dataset.id))
            .filter(datasets::global_id.eq(ALA_DATASET_ID))
            .select(taxa::all_columns)
            .order_by(taxa::scientific_name)
            .paginate(page)
            .load::<(Taxon, i64)>(&mut conn)
            .await?;

        Ok(species.into())
    }
}
