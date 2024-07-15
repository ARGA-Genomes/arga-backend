use std::collections::HashMap;

use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type DnaExtractMap = HashMap<String, DnaExtractMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnaExtractRecord {
    pub record_id: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct DnaExtractMatch {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub record_id: String,
}


pub fn dna_extract_map(datasets: &Vec<Uuid>, pool: &mut PgPool) -> Result<DnaExtractMap, Error> {
    use schema::dna_extracts::dsl::*;
    info!("Creating dna extract map");

    let mut conn = pool.get()?;

    let results = dna_extracts
        .select((id, dataset_id, name_id, record_id))
        .filter(dataset_id.eq_any(datasets))
        .load::<DnaExtractMatch>(&mut conn)?;

    let mut map = DnaExtractMap::new();
    for dna_extract_match in results {
        map.insert(dna_extract_match.record_id.clone(), dna_extract_match);
    }

    info!(total = map.len(), "Creating dna extract map finished");
    Ok(map)
}


pub fn match_records_mapped<T>(records: Vec<T>, subsamples: &DnaExtractMap) -> Vec<(DnaExtractMatch, T)>
where
    T: Clone + Into<DnaExtractRecord>,
{
    // associate the records with the matched name
    let mut matched: Vec<(DnaExtractMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let dna_extract_record = record.clone().into();

        if let Some(extract) = subsamples.get(&dna_extract_record.record_id) {
            matched.push((extract.clone(), record));
        }
    }

    matched
}
