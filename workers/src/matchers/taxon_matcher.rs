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


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonRecord {
    pub scientific_name: String,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct TaxonMatch {
    pub id: Uuid,
    pub scientific_name: String,
}


pub fn match_taxa(records: &Vec<TaxonRecord>, pool: &mut PgPool) -> HashMap<String, TaxonMatch> {
    use schema::taxa::dsl::*;
    info!(total=records.len(), "Matching taxa");

    // get 50,000 possible matches in parallel. this can speed up the matching significantly
    // since our main limit here is the parameter limit in postgres
    let matched: Vec<Result<Vec<TaxonMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let all_names: Vec<&String> = chunk.iter().map(|row| &row.scientific_name).collect();

        let results = taxa
            .select((id, scientific_name))
            .filter(scientific_name.eq_any(&all_names))
            .load::<TaxonMatch>(&mut conn)?;

        Ok::<Vec<TaxonMatch>, Error>(results)
    }).collect();

    let mut map: HashMap<String, TaxonMatch> = HashMap::new();

    for chunk in matched {
        if let Ok(matches) = chunk {
            for taxon_match in matches {
                map.insert(taxon_match.scientific_name.clone(), taxon_match);
            }
        }
    }

    info!(total=records.len(), matched=map.len(), "Matching taxa finished");
    map
}
