use std::collections::HashMap;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::schema;
use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type DatasetMap = HashMap<String, DatasetMatch>;


#[derive(Debug, Deserialize)]
pub struct DatasetRecord {
    pub global_id: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct DatasetMatch {
    pub id: Uuid,
    pub global_id: String,
}


pub fn match_datasets<T>(records: &Vec<T>, pool: &mut PgPool) -> DatasetMap
where T: Sync + Clone + Into<DatasetRecord>
{
    use schema::datasets::dsl::*;
    info!(total=records.len(), "Matching datasets");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<DatasetMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let all_ids: Vec<String> = chunk.iter().map(|row| row.clone().into().global_id).collect();

        let results = datasets
            .select((id, global_id))
            .filter(global_id.eq_any(&all_ids))
            .load::<DatasetMatch>(&mut conn)?;

        Ok::<Vec<DatasetMatch>, Error>(results)
    }).collect();

    let mut map = DatasetMap::new();

    for chunk in matched {
        if let Ok(matches) = chunk {
            for dataset_match in matches {
                map.insert(dataset_match.global_id.clone(), dataset_match);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching datasets finished");
    map
}


pub fn dataset_map(pool: &mut PgPool) -> Result<DatasetMap, Error> {
    use schema::datasets::dsl::*;
    info!("Creating dataset map");

    let mut conn = pool.get()?;

    let results = datasets
        .select((id, global_id))
        .load::<DatasetMatch>(&mut conn)?;

    let mut map = DatasetMap::new();
    for dataset_match in results {
        map.insert(dataset_match.global_id.clone(), dataset_match);
    }

    info!(total=map.len(), "Creating dataset map finished");
    Ok(map)
}


pub fn match_records<T>(records: Vec<T>, datasets: &DatasetMap) -> Vec<(DatasetMatch, T)>
where T: Clone + Into<DatasetRecord>
{
    // associate the records with the matched name
    let mut matched: Vec<(DatasetMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let sequence_record = record.clone().into();

        if let Some(sequence) = datasets.get(&sequence_record.global_id) {
            matched.push((sequence.clone(), record));
        }
    }

    matched
}
