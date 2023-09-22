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


pub fn specimen_map(dataset: &Uuid, pool: &mut PgPool) -> Result<SpecimenMap, Error> {
    use schema::specimens::dsl::*;
    info!("Creating specimen map");

    let mut conn = pool.get()?;

    let results = specimens
        .select((id, dataset_id, name_id, record_id))
        .filter(dataset_id.eq(dataset))
        .load::<SpecimenMatch>(&mut conn)?;

    let mut map = SpecimenMap::new();
    for specimen_match in results {
        map.insert(specimen_match.record_id.clone(), specimen_match);
    }

    info!(total=map.len(), "Creating specimen map finished");
    Ok(map)
}

pub fn match_specimens(records: &Vec<SpecimenRecord>, dataset: &Uuid, pool: &mut PgPool) -> HashMap<String, SpecimenMatch> {
    use schema::specimens::dsl::*;
    info!(total=records.len(), "Matching specimens");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<SpecimenMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let accessions: Vec<&String> = chunk.iter().map(|row| &row.record_id).collect();

        let results = specimens
            .select((id, dataset_id, name_id, record_id))
            .filter(dataset_id.eq(&dataset))
            .filter(record_id.eq_any(&accessions))
            .load::<SpecimenMatch>(&mut conn)?;

        Ok::<Vec<SpecimenMatch>, Error>(results)
    }).collect();

    let mut map: HashMap<String, SpecimenMatch> = HashMap::new();

    for chunk in matched {
        if let Ok(records) = chunk {
            for record in records {
                map.insert(record.record_id.clone(), record);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching specimens finished");
    map
}


pub fn match_records<T>(records: Vec<T>, dataset_id: &Uuid, pool: &mut PgPool) -> Vec<(SpecimenMatch, T)>
where T: Clone + Into<SpecimenRecord>
{
    // convert the records into specimen records for matching
    let mut specimen_records: Vec<SpecimenRecord> = Vec::with_capacity(records.len());
    for record in &records {
        specimen_records.push(record.clone().into());
    }

    // get the match for each record from the database
    let specimens = match_specimens(&specimen_records, dataset_id, pool);
    match_records_mapped(records, &specimens)
}


pub fn match_records_mapped<T>(records: Vec<T>, specimens: &SpecimenMap) -> Vec<(SpecimenMatch, T)>
where T: Clone + Into<SpecimenRecord>
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
