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
pub type NameMap = HashMap<String, NameMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameRecord {
    pub scientific_name: Option<String>,
    pub canonical_name: Option<String>,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct NameMatch {
    pub id: Uuid,
    pub scientific_name: String,
    pub canonical_name: String,
}


pub fn name_map(pool: &mut PgPool) -> Result<NameMap, Error> {
    use schema::names::dsl::*;
    info!("Creating name map");

    let mut conn = pool.get()?;

    let results = names
        .select((id, scientific_name, canonical_name))
        .load::<NameMatch>(&mut conn)?;

    let mut map = NameMap::new();
    for name_match in results {
        map.insert(name_match.scientific_name.clone(), name_match.clone());
        map.insert(name_match.canonical_name.clone(), name_match);
    }

    info!(total=map.len(), "Creating name map finished");
    Ok(map)
}

pub fn match_names(records: &Vec<NameRecord>, pool: &mut PgPool) -> HashMap<String, NameMatch> {
    use schema::names;
    info!(total=records.len(), "Matching names");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<NameMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let scientific_names: Vec<&String> = chunk.iter().filter_map(|row| row.scientific_name.as_ref()).collect();
        let canonical_names: Vec<&String> = chunk.iter().filter_map(|row| row.canonical_name.as_ref()).collect();

        let mut all_names = scientific_names;
        all_names.extend(canonical_names);

        let results = names::table
            .select((names::id, names::scientific_name, names::canonical_name))
            .filter(names::scientific_name.eq_any(&all_names))
            .or_filter(names::canonical_name.eq_any(&all_names))
            .load::<NameMatch>(&mut conn)?;

        Ok::<Vec<NameMatch>, Error>(results)
    }).collect();

    let mut map: HashMap<String, NameMatch> = HashMap::new();

    // for every bulk name match result from the database we insert it into a map
    // that can be referenced either by scientific name or canonical name as some
    // scenarios only have canonical names available
    for chunk in matched {
        if let Ok(names) = chunk {
            for name_match in names {
                map.insert(name_match.canonical_name.clone(), name_match.clone());
                map.insert(name_match.scientific_name.clone(), name_match);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching names finished");
    map
}


pub fn match_records<T>(records: Vec<T>, pool: &mut PgPool) -> Vec<(NameMatch, T)>
where T: Clone + Into<NameRecord>
{
    // convert the records into name records for matching
    let mut name_records: Vec<NameRecord> = Vec::with_capacity(records.len());
    for record in &records {
        name_records.push(record.clone().into());
    }

    // get the name match for each record from the database
    let names = match_names(&name_records, pool);

    // associate the records with the matched name
    let mut matched: Vec<(NameMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let name_record = record.clone().into();

        // try to match on scientific name first and fall back to matching by
        // canonical name if not provided. importantly this does not try to match
        // by canonical name if a match for scientific name was not found. it only
        // tries an alternative column to match on as scientific names are often not
        // provided.
        // If scientific names are provided then it will only attempt to match on that.
        if let Some(scientific_name) = name_record.scientific_name {
            if let Some(name) = names.get(&scientific_name) {
                matched.push((name.clone(), record));
            }
        }
        else if let Some(canonical_name) = name_record.canonical_name {
            if let Some(name) = names.get(&canonical_name) {
                matched.push((name.clone(), record));
            }
        }
    }

    matched
}


pub fn match_records_mapped<T>(records: Vec<T>, names: &NameMap) -> Result<Vec<(NameMatch, T)>, Error>
where T: Clone + Into<NameRecord>
{
    // associate the records with the matched name
    let mut matched: Vec<(NameMatch, T)> = Vec::with_capacity(records.len());
    for record in records {
        let name_record = record.clone().into();

        // try to match on scientific name first and fall back to matching by
        // canonical name if not provided. importantly this does not try to match
        // by canonical name if a match for scientific name was not found. it only
        // tries an alternative column to match on as scientific names are often not
        // provided.
        // If scientific names are provided then it will only attempt to match on that.
        if let Some(scientific_name) = name_record.scientific_name {
            if let Some(name) = names.get(&scientific_name) {
                matched.push((name.clone(), record));
            }
        }
        else if let Some(canonical_name) = name_record.canonical_name {
            if let Some(name) = names.get(&canonical_name) {
                matched.push((name.clone(), record));
            }
        }
    }

    Ok(matched)
}
