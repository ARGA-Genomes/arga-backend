use std::collections::HashMap;

use arga_core::models::Dataset;
use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonRecord {
    pub scientific_name: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct TaxonMatch {
    pub id: Uuid,
    pub scientific_name: String,
}


pub fn taxa_map(dataset: &Dataset, pool: &mut PgPool) -> Result<HashMap<String, TaxonMatch>, Error> {
    use schema::taxa::dsl::*;
    let mut conn = pool.get()?;

    info!("Generating taxa map");

    let records = taxa
        .select((id, scientific_name))
        .filter(dataset_id.eq(dataset.id))
        .load::<TaxonMatch>(&mut conn)?;

    let mut map = HashMap::new();
    for record in records.into_iter() {
        map.insert(record.scientific_name.clone(), record);
    }

    info!(total = map.len(), "Generating taxa map finished");
    Ok(map)
}
