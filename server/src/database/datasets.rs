use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::models::{Dataset, Taxon};
use super::{schema, PageResult, PgPool};
use crate::database::Error;


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

    pub async fn species(&self, _dataset: &Dataset, _page: i64) -> PageResult<Taxon> {
        // use schema::{indigenous_knowledge as iek, taxa};
        // let mut conn = self.pool.get().await?;

        // join the taxa table with all dataset tables to filter get taxonomy
        // of species that appear in a dataset.
        // FIXME: find new pathway linking name_attributes to taxa via taxon_names
        let species = vec![];
        // let species = taxa::table
        //     .left_join(iek::table.on(taxa::name_id.eq(iek::name_id)))
        //     .filter(iek::dataset_id.eq(dataset.id))
        //     .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
        //     .select(taxa::all_columns)
        //     .order_by(taxa::scientific_name)
        //     .paginate(page)
        //     .load::<(Taxon, i64)>(&mut conn)
        //     .await?;

        Ok(species.into())
    }
}
