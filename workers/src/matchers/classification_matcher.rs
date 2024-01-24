use std::collections::HashMap;

use arga_core::models::{TaxonomicStatus, Taxon};
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
    pub scientific_name: String,
    pub canonical_name: String,
    pub status: TaxonomicStatus,
}


pub fn classification_map(pool: &mut PgPool) -> Result<ClassificationMap, Error> {
    use schema::taxa::dsl::*;
    info!("Creating classification map");

    let mut conn = pool.get()?;

    let results = taxa
        .select((id, scientific_name, canonical_name, status))
        .load::<ClassificationMatch>(&mut conn)?;

    let mut map = ClassificationMap::new();
    for classification_match in results {
        map.insert(classification_match.scientific_name.clone(), classification_match.clone());
    }

    info!(total=map.len(), "Creating classification map finished");
    Ok(map)
}
