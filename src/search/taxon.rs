use serde::Deserialize;
use serde::Serialize;

use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

use uuid::Uuid;

use crate::database::models::TaxonomicStatus;
use crate::database::{schema_gnl, Database};
use crate::http::Error;


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct SpeciesDoc {
    pub name_id: Uuid,
    pub canonical_name: Option<String>,
    pub subspecies: Option<Vec<String>>,
    pub synonyms: Option<Vec<String>>,
}

pub async fn get_species(db: &Database) -> Result<Vec<SpeciesDoc>, Error> {
    use schema_gnl::{species, synonyms};
    let mut conn = db.pool.get().await.unwrap();

    let docs = species::table
        .left_join(synonyms::table)
        .select((
            species::name_id,
            species::canonical_name,
            species::subspecies,
            synonyms::names.nullable(),
        ))
        .filter(species::status.eq(TaxonomicStatus::Valid))
        .load::<SpeciesDoc>(&mut conn)
        .await?;

    Ok(docs)
}


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct UndescribedSpeciesDoc {
    pub genus: String,
    pub genus_authority: Option<String>,
    pub names: Vec<String>,
}

pub async fn get_undescribed_species(db: &Database) -> Result<Vec<UndescribedSpeciesDoc>, Error> {
    use schema_gnl::undescribed_species::dsl::*;
    let mut conn = db.pool.get().await.unwrap();

    let docs = undescribed_species
        .load::<UndescribedSpeciesDoc>(&mut conn)
        .await?;

    Ok(docs)
}
