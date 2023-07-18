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
    pub status: TaxonomicStatus,

    pub canonical_name: Option<String>,
    pub subspecies: Option<Vec<String>>,
    pub synonyms: Option<Vec<String>>,
    pub vernacular_names: Option<Vec<String>>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
}

pub async fn get_species(db: &Database) -> Result<Vec<SpeciesDoc>, Error> {
    use schema_gnl::{species, synonyms, species_vernacular_names};
    let mut conn = db.pool.get().await.unwrap();

    let docs = species::table
        .left_join(synonyms::table)
        .left_join(species_vernacular_names::table)
        .select((
            species::name_id,
            species::status,

            species::canonical_name,
            species::subspecies,
            synonyms::names.nullable(),
            species_vernacular_names::vernacular_names.nullable(),

            species::kingdom,
            species::phylum,
            species::class,
            species::order,
            species::family,
            species::genus,
        ))
        .filter(species::status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<SpeciesDoc>(&mut conn)
        .await?;

    Ok(docs)
}
