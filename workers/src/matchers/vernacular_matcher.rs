use std::collections::HashMap;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;

use arga_core::schema;
use crate::error::Error;

use super::name_matcher::{NameMatch, self, NameRecord};

type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VernacularRecord {
    pub scientific_name: String,
    pub canonical_name: Option<String>,

    pub vernacular_name: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct VernacularMatch {
    pub id: i64,
    pub vernacular_name: String,
    pub language: Option<String>,
}


pub fn match_vernacular(records: &Vec<VernacularRecord>, pool: &mut PgPool) -> HashMap<String, VernacularMatch> {
    use schema::vernacular_names::dsl::*;
    info!(total=records.len(), "Matching vernacular names");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<VernacularMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let all_names: Vec<&String> = chunk.iter().map(|row| &row.vernacular_name).collect();

        let results = vernacular_names
            .select((id, vernacular_name, language))
            .filter(vernacular_name.eq_any(&all_names))
            .load::<VernacularMatch>(&mut conn)?;

        Ok::<Vec<VernacularMatch>, Error>(results)
    }).collect();

    let mut map: HashMap<String, VernacularMatch> = HashMap::new();

    // generate a map of vernacular name ids so that we can link to the one
    // copy of the name via vernacular links
    for chunk in matched {
        if let Ok(names) = chunk {
            for name_match in names {
                map.insert(name_match.vernacular_name.clone(), name_match);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching names finished");
    map
}


/// Matches records and its accompanying names to a vernacular name record.
///
/// This requires the record to be convertable into a VernacularRecord as well as a NameRecord
/// because it will execute the name matcher to associate the records with both vernacular and name
/// records. More often than not we are going to want both since a vernacular record is pretty much
/// only associated with a name.
pub fn match_records<T>(records: Vec<T>, pool: &mut PgPool) -> Vec<(VernacularMatch, NameMatch, T)>
where T: Clone + Into<VernacularRecord> + Into<NameRecord>
{
    let records = name_matcher::match_records(records, pool);

    // convert the records into vernacular records for matching
    let mut vernacular_records: Vec<VernacularRecord> = Vec::with_capacity(records.len());
    for (_name, record) in &records {
        vernacular_records.push(record.clone().into());
    }

    // get the vernacular match for each record from the database
    let names = match_vernacular(&vernacular_records, pool);

    // associate the records with the matched vernacular name
    let mut matched: Vec<(VernacularMatch, NameMatch, T)> = Vec::with_capacity(records.len());
    for (name_match, record) in records {
        let vernacular_record: VernacularRecord = record.clone().into();
        if let Some(vernacular_match) = names.get(&vernacular_record.vernacular_name) {
            matched.push((vernacular_match.clone(), name_match, record));
        }
    }

    matched
}
