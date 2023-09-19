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
pub type DnaExtractMap = HashMap<String, DnaExtractMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnaExtractRecord {
    pub accession: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct DnaExtractMatch {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub accession: String,
}


pub fn dna_extract_map(datasets: &Vec<Uuid>, pool: &mut PgPool) -> Result<DnaExtractMap, Error> {
    use schema::dna_extracts::dsl::*;
    info!("Creating dna extract map");

    let mut conn = pool.get()?;

    let results = dna_extracts
        .select((id, dataset_id, name_id, accession))
        .filter(dataset_id.eq_any(datasets))
        .load::<DnaExtractMatch>(&mut conn)?;

    let mut map = DnaExtractMap::new();
    for dna_extract_match in results {
        map.insert(dna_extract_match.accession.clone(), dna_extract_match);
    }

    info!(total=map.len(), "Creating dna extract map finished");
    Ok(map)
}

pub fn match_dna_extracts(records: &Vec<DnaExtractRecord>, dataset: &Uuid, pool: &mut PgPool) -> HashMap<String, DnaExtractMatch> {
    use schema::dna_extracts::dsl::*;
    info!(total=records.len(), "Matching dna extracts");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<DnaExtractMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let accessions: Vec<&String> = chunk.iter().map(|row| &row.accession).collect();

        let results = dna_extracts
            .select((id, dataset_id, name_id, accession))
            .filter(dataset_id.eq(&dataset))
            .filter(accession.eq_any(&accessions))
            .load::<DnaExtractMatch>(&mut conn)?;

        Ok::<Vec<DnaExtractMatch>, Error>(results)
    }).collect();

    let mut map: HashMap<String, DnaExtractMatch> = HashMap::new();

    for chunk in matched {
        if let Ok(records) = chunk {
            for record in records {
                map.insert(record.accession.clone(), record);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching dna extracts finished");
    map
}


pub fn match_records<T>(records: Vec<T>, dataset_id: &Uuid, pool: &mut PgPool) -> Vec<(DnaExtractMatch, T)>
where T: Clone + Into<DnaExtractRecord>
{
    // convert the records into dna extract records for matching
    let mut dna_extract_records: Vec<DnaExtractRecord> = Vec::with_capacity(records.len());
    for record in &records {
        dna_extract_records.push(record.clone().into());
    }

    // get the match for each record from the database
    let extracts = match_dna_extracts(&dna_extract_records, dataset_id, pool);
    match_records_mapped(records, &extracts)
}


pub fn match_records_mapped<T>(records: Vec<T>, subsamples: &DnaExtractMap) -> Vec<(DnaExtractMatch, T)>
where T: Clone + Into<DnaExtractRecord>
{
    // associate the records with the matched name
    let mut matched: Vec<(DnaExtractMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let dna_extract_record = record.clone().into();

        if let Some(extract) = subsamples.get(&dna_extract_record.accession) {
            matched.push((extract.clone(), record));
        }
    }

    matched
}
