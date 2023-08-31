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
pub type SourceMap = HashMap<String, SourceMatch>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRecord {
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct SourceMatch {
    pub id: Uuid,
    pub name: String,
}


pub fn match_sources<T>(records: &Vec<T>, pool: &mut PgPool) -> SourceMap
where T: Sync + Clone + Into<SourceRecord>
{
    use schema::sources::dsl::*;
    info!(total=records.len(), "Matching sources");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<SourceMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let all_names: Vec<String> = chunk.iter().map(|row| row.clone().into().name).collect();

        let results = sources
            .select((id, name))
            .filter(name.eq_any(&all_names))
            .load::<SourceMatch>(&mut conn)?;

        Ok::<Vec<SourceMatch>, Error>(results)
    }).collect();

    let mut map = SourceMap::new();

    for chunk in matched {
        if let Ok(matches) = chunk {
            for source_match in matches {
                map.insert(source_match.name.clone(), source_match);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching sources finished");
    map
}
