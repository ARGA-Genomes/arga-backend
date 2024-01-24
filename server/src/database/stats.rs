use arga_core::models::Dataset;
use async_graphql::SimpleObject;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};

use super::{schema, Error, PgPool};


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetStats {
    /// The total amount of species in the order
    pub total_species: usize,
    pub total_species_with_data: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetBreakdown {
    pub species: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownItem {
    pub name: Option<String>,
    pub total: i64,
}


#[derive(Clone)]
pub struct StatsProvider {
    pub pool: PgPool,
}

impl StatsProvider {
    pub async fn dataset(&self, name: &str) -> Result<DatasetStats, Error> {
        use schema::{datasets, names, indigenous_knowledge as iek};
        let mut conn = self.pool.get().await?;

        let dataset = datasets::table
            .filter(datasets::name.eq(&name))
            .get_result::<Dataset>(&mut conn)
            .await?;

        let total: i64 = names::table
            .left_join(iek::table.on(names::id.eq(iek::name_id)))
            .filter(iek::dataset_id.eq(dataset.id))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(DatasetStats {
            // this can never be negative due to the count
            total_species: total as usize,
            total_species_with_data: 0,
        })
    }

    pub async fn dataset_breakdown(&self, _name: &str) -> Result<DatasetBreakdown, Error> {
        let species = vec![];

        Ok(DatasetBreakdown {
            species,
        })
    }
}
