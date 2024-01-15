use std::collections::HashMap;

use arga_core::models::{TaxonomicStatus, Classification};
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use arga_core::schema;
use crate::error::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type ClassificationMap = HashMap<String, ClassificationMatch>;
pub type TaxaMap = HashMap<String, Classification>;


#[derive(Debug, Deserialize)]
pub struct ClassificationRecord {
    pub taxon_id: Option<String>,
    pub scientific_name: Option<String>,
    pub canonical_name: Option<String>,
}

#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct ClassificationMatch {
    pub id: Uuid,
    pub taxon_id: i32,
    pub scientific_name: String,
    pub canonical_name: String,
    pub status: TaxonomicStatus,
}


pub fn classification_map(pool: &mut PgPool) -> Result<ClassificationMap, Error> {
    use schema::classifications::dsl::*;
    info!("Creating classification map");

    let mut conn = pool.get()?;

    let results = classifications
        .select((id, taxon_id, scientific_name, canonical_name, status))
        .load::<ClassificationMatch>(&mut conn)?;

    let mut map = ClassificationMap::new();
    for classification_match in results {
        map.insert(classification_match.taxon_id.to_string(), classification_match.clone());
        map.insert(classification_match.scientific_name.clone(), classification_match.clone());
        // map.insert(classification_match.canonical_name.clone(), classification_match.clone());
    }

    info!(total=map.len(), "Creating classification map finished");
    Ok(map)
}


pub fn taxa_map(pool: &mut PgPool) -> Result<TaxaMap, Error> {
    use schema::classifications::dsl::*;
    info!("Creating taxa map");

    let mut conn = pool.get()?;

    let results = classifications.load::<Classification>(&mut conn)?;

    let mut map = TaxaMap::new();
    for taxon in results {
        map.insert(taxon.taxon_id.to_string(), taxon.clone());
        map.insert(taxon.scientific_name.clone(), taxon.clone());
        map.insert(taxon.canonical_name.clone(), taxon.clone());
    }

    info!(total=map.len(), "Creating taxa map finished");
    Ok(map)
}
