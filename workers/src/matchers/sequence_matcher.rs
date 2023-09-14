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
pub type SequenceMap = HashMap<String, SequenceMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceRecord {
    pub accession: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct SequenceMatch {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub accession: String,
}


pub fn sequence_map(dataset: &Uuid, pool: &mut PgPool) -> Result<SequenceMap, Error> {
    use schema::sequences::dsl::*;
    info!("Creating sequence map");

    let mut conn = pool.get()?;

    let results = sequences
        .select((id, dataset_id, name_id, accession))
        .filter(dataset_id.eq(dataset))
        .load::<SequenceMatch>(&mut conn)?;

    let mut map = SequenceMap::new();
    for sequence_match in results {
        map.insert(sequence_match.accession.clone(), sequence_match);
    }

    info!(total=map.len(), "Creating sequence map finished");
    Ok(map)
}

pub fn match_sequences(records: &Vec<SequenceRecord>, dataset: &Uuid, pool: &mut PgPool) -> HashMap<String, SequenceMatch> {
    use schema::sequences::dsl::*;
    info!(total=records.len(), "Matching sequences");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<SequenceMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let accessions: Vec<&String> = chunk.iter().map(|row| &row.accession).collect();

        let results = sequences
            .select((id, dataset_id, name_id, accession))
            .filter(dataset_id.eq(&dataset))
            .filter(accession.eq_any(&accessions))
            .load::<SequenceMatch>(&mut conn)?;

        Ok::<Vec<SequenceMatch>, Error>(results)
    }).collect();

    let mut map: HashMap<String, SequenceMatch> = HashMap::new();

    for chunk in matched {
        if let Ok(records) = chunk {
            for record in records {
                map.insert(record.accession.clone(), record);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching sequences finished");
    map
}


pub fn match_records<T>(records: Vec<T>, dataset_id: &Uuid, pool: &mut PgPool) -> Vec<(SequenceMatch, T)>
where T: Clone + Into<SequenceRecord>
{
    // convert the records into subsample records for matching
    let mut sequence_records: Vec<SequenceRecord> = Vec::with_capacity(records.len());
    for record in &records {
        sequence_records.push(record.clone().into());
    }

    // get the match for each record from the database
    let sequences = match_sequences(&sequence_records, dataset_id, pool);
    match_records_mapped(records, &sequences)
}


pub fn match_records_mapped<T>(records: Vec<T>, sequences: &SequenceMap) -> Vec<(SequenceMatch, T)>
where T: Clone + Into<SequenceRecord>
{
    // associate the records with the matched name
    let mut matched: Vec<(SequenceMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let sequence_record = record.clone().into();

        if let Some(sequence) = sequences.get(&sequence_record.accession) {
            matched.push((sequence.clone(), record));
        }
    }

    matched
}
