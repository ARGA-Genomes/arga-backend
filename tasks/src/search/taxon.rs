use serde::Deserialize;
use serde::Serialize;

use diesel::*;
use diesel::RunQueryDsl;
use diesel::r2d2::{ConnectionManager, Pool};

use uuid::Uuid;
use anyhow::Error;

use arga_core::models::TaxonomicStatus;
use arga_core::{schema, schema_gnl};


type PgPool = Pool<ConnectionManager<PgConnection>>;


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

pub fn get_species(pool: &PgPool) -> Result<Vec<SpeciesDoc>, Error> {
    use schema_gnl::{species, synonyms, species_vernacular_names};
    let mut conn = pool.get()?;

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
        .filter(species::status.eq_any(&[TaxonomicStatus::Valid]))
        .load::<SpeciesDoc>(&mut conn)?;

    Ok(docs)
}

pub fn get_undescribed_species(pool: &PgPool) -> Result<Vec<SpeciesDoc>, Error> {
    use schema::taxa;
    use schema_gnl::{species, synonyms, species_vernacular_names};
    let mut conn = pool.get()?;

    let docs = taxa::table
        .left_join(species::table)
        .left_join(synonyms::table)
        .left_join(species_vernacular_names::table)
        .select((
            taxa::name_id,
            taxa::status,

            taxa::canonical_name,
            species::subspecies.nullable(),
            synonyms::names.nullable(),
            species_vernacular_names::vernacular_names.nullable(),

            taxa::kingdom,
            taxa::phylum,
            taxa::class,
            taxa::order,
            taxa::family,
            taxa::genus,
        ))
        .filter(taxa::status.eq_any(&[TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<SpeciesDoc>(&mut conn)?;

    Ok(docs)
}
