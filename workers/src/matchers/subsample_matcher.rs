use std::collections::HashMap;

use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type SubsampleMap = HashMap<String, SubsampleMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsampleRecord {
    pub record_id: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct SubsampleMatch {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub record_id: String,
}


pub fn subsample_map(datasets: &Vec<Uuid>, pool: &mut PgPool) -> Result<SubsampleMap, Error> {
    use schema::subsamples::dsl::*;
    info!("Creating subsample map");

    let mut conn = pool.get()?;

    let results = subsamples
        .select((id, dataset_id, name_id, record_id))
        .filter(dataset_id.eq_any(datasets))
        .load::<SubsampleMatch>(&mut conn)?;

    let mut map = SubsampleMap::new();
    for subsample_match in results {
        map.insert(subsample_match.record_id.clone(), subsample_match);
    }

    info!(total = map.len(), "Creating subsample map finished");
    Ok(map)
}


pub fn match_records_mapped<T>(records: Vec<T>, subsamples: &SubsampleMap) -> Vec<(SubsampleMatch, T)>
where
    T: Clone + Into<SubsampleRecord>,
{
    // associate the records with the matched name
    let mut matched: Vec<(SubsampleMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let subsample_record = record.clone().into();

        if let Some(subsample) = subsamples.get(&subsample_record.record_id) {
            matched.push((subsample.clone(), record));
        }
    }

    matched
}
