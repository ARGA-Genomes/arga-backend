use std::collections::HashMap;

use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type SpecimenMap = HashMap<String, SpecimenMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecimenRecord {
    pub record_id: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct SpecimenMatch {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub record_id: String,
}


pub fn specimen_map(datasets: &Vec<Uuid>, pool: &mut PgPool) -> Result<SpecimenMap, Error> {
    use schema::specimens::dsl::*;
    info!("Creating specimen map");

    let mut conn = pool.get()?;

    let results = specimens
        .select((id, dataset_id, name_id, record_id))
        .filter(dataset_id.eq_any(datasets))
        .load::<SpecimenMatch>(&mut conn)?;

    let mut map = SpecimenMap::new();
    for specimen_match in results {
        map.insert(specimen_match.record_id.clone(), specimen_match);
    }

    info!(total = map.len(), "Creating specimen map finished");
    Ok(map)
}


pub fn match_records_mapped<T>(records: Vec<T>, specimens: &SpecimenMap) -> Vec<(SpecimenMatch, T)>
where
    T: Clone + Into<SpecimenRecord>,
{
    // associate the records with the matched name
    let mut matched: Vec<(SpecimenMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let specimen_record = record.clone().into();

        if let Some(specimen) = specimens.get(&specimen_record.record_id) {
            matched.push((specimen.clone(), record));
        }
    }

    matched
}
