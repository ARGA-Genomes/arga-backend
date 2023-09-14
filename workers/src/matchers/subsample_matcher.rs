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
pub type SubsampleMap = HashMap<String, SubsampleMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsampleRecord {
    pub accession: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct SubsampleMatch {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub accession: String,
}


pub fn subsample_map(dataset: &Uuid, pool: &mut PgPool) -> Result<SubsampleMap, Error> {
    use schema::subsamples::dsl::*;
    info!("Creating subsample map");

    let mut conn = pool.get()?;

    let results = subsamples
        .select((id, dataset_id, name_id, accession))
        .filter(dataset_id.eq(dataset))
        .load::<SubsampleMatch>(&mut conn)?;

    let mut map = SubsampleMap::new();
    for subsample_match in results {
        map.insert(subsample_match.accession.clone(), subsample_match);
    }

    info!(total=map.len(), "Creating subsample map finished");
    Ok(map)
}

pub fn match_subsamples(records: &Vec<SubsampleRecord>, dataset: &Uuid, pool: &mut PgPool) -> HashMap<String, SubsampleMatch> {
    use schema::subsamples::dsl::*;
    info!(total=records.len(), "Matching subsamples");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<SubsampleMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let accessions: Vec<&String> = chunk.iter().map(|row| &row.accession).collect();

        let results = subsamples
            .select((id, dataset_id, name_id, accession))
            .filter(dataset_id.eq(&dataset))
            .filter(accession.eq_any(&accessions))
            .load::<SubsampleMatch>(&mut conn)?;

        Ok::<Vec<SubsampleMatch>, Error>(results)
    }).collect();

    let mut map: HashMap<String, SubsampleMatch> = HashMap::new();

    for chunk in matched {
        if let Ok(records) = chunk {
            for record in records {
                map.insert(record.accession.clone(), record);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching subsamples finished");
    map
}


pub fn match_records<T>(records: Vec<T>, dataset_id: &Uuid, pool: &mut PgPool) -> Vec<(SubsampleMatch, T)>
where T: Clone + Into<SubsampleRecord>
{
    // convert the records into subsample records for matching
    let mut subsample_records: Vec<SubsampleRecord> = Vec::with_capacity(records.len());
    for record in &records {
        subsample_records.push(record.clone().into());
    }

    // get the match for each record from the database
    let subsamples = match_subsamples(&subsample_records, dataset_id, pool);
    match_records_mapped(records, &subsamples)
}


pub fn match_records_mapped<T>(records: Vec<T>, subsamples: &SubsampleMap) -> Vec<(SubsampleMatch, T)>
where T: Clone + Into<SubsampleRecord>
{
    // associate the records with the matched name
    let mut matched: Vec<(SubsampleMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let subsample_record = record.clone().into();

        if let Some(subsample) = subsamples.get(&subsample_record.accession) {
            matched.push((subsample.clone(), record));
        }
    }

    matched
}
