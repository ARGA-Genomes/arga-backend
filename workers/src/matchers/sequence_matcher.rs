use std::collections::HashMap;

use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type SequenceMap = HashMap<String, SequenceMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceRecord {
    pub record_id: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct SequenceMatch {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub record_id: String,
}


pub fn sequence_map(datasets: &Vec<Uuid>, pool: &mut PgPool) -> Result<SequenceMap, Error> {
    use schema::sequences::dsl::*;
    info!("Creating sequence map");

    let mut conn = pool.get()?;

    let results = sequences
        .select((id, dataset_id, name_id, record_id))
        .filter(dataset_id.eq_any(datasets))
        .load::<SequenceMatch>(&mut conn)?;

    let mut map = SequenceMap::new();
    for sequence_match in results {
        map.insert(sequence_match.record_id.clone(), sequence_match);
    }

    info!(total = map.len(), "Creating sequence map finished");
    Ok(map)
}


pub fn match_records_mapped<T>(records: Vec<T>, sequences: &SequenceMap) -> Vec<(SequenceMatch, T)>
where
    T: Clone + Into<SequenceRecord>,
{
    // associate the records with the matched name
    let mut matched: Vec<(SequenceMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let sequence_record = record.clone().into();

        if let Some(sequence) = sequences.get(&sequence_record.record_id) {
            matched.push((sequence.clone(), record));
        }
    }

    matched
}
