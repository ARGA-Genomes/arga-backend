use std::collections::HashMap;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::schema;
use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type ClassificationMap = HashMap<String, ClassificationMatch>;


#[derive(Debug, Deserialize)]
pub struct ClassificationRecord {
    pub taxon_id: Option<String>,
    pub scientific_name: Option<String>,
    pub canonical_name: Option<String>,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct ClassificationMatch {
    pub id: Uuid,
    pub taxon_id: String,
    pub scientific_name: String,
    pub canonical_name: String,
}


pub fn classification_map(pool: &mut PgPool) -> Result<ClassificationMap, Error> {
    use schema::classifications::dsl::*;
    info!("Creating classification map");

    let mut conn = pool.get()?;

    let results = classifications
        .select((id, taxon_id, scientific_name, canonical_name))
        .load::<ClassificationMatch>(&mut conn)?;

    let mut map = ClassificationMap::new();
    for classification_match in results {
        map.insert(classification_match.taxon_id.clone(), classification_match.clone());
        map.insert(classification_match.scientific_name.clone(), classification_match.clone());
        map.insert(classification_match.canonical_name.clone(), classification_match);
    }

    info!(total=map.len(), "Creating classification map finished");
    Ok(map)
}
