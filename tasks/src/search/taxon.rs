use diesel::sql_types::Nullable;
use diesel::sql_types::Varchar;
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

    pub canonical_name: String,
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
    use diesel::dsl::sql;
    use schema_gnl::{species, synonyms, common_names};
    let mut conn = pool.get()?;

    let docs = species::table
        .left_join(synonyms::table)
        .left_join(common_names::table)
        .select((
            species::name_id,
            species::status,

            species::canonical_name,
            species::subspecies,
            synonyms::names.nullable(),
            common_names::names.nullable(),

            sql::<Nullable<Varchar>>("classification->>'kingdom'"),
            sql::<Nullable<Varchar>>("classification->>'phylum'"),
            sql::<Nullable<Varchar>>("classification->>'class'"),
            sql::<Nullable<Varchar>>("classification->>'order'"),
            sql::<Nullable<Varchar>>("classification->>'family'"),
            sql::<Nullable<Varchar>>("classification->>'genus'"),
        ))
        .filter(species::status.eq_any(&[TaxonomicStatus::Accepted]))
        .load::<SpeciesDoc>(&mut conn)?;

    Ok(docs)
}



#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct UndescribedSpeciesDoc {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,

    pub canonical_name: String,
    pub subspecies: Option<Vec<String>>,
    pub synonyms: Option<Vec<String>>,
    pub vernacular_names: Option<Vec<String>>,
}

pub fn get_undescribed_species(pool: &PgPool) -> Result<Vec<SpeciesDoc>, Error> {
    use schema::taxa;
    use schema_gnl::{species, synonyms, common_names};
    let mut conn = pool.get()?;

    let docs = taxa::table
        .left_join(species::table)
        .left_join(synonyms::table)
        .left_join(common_names::table)
        .select((
            taxa::name_id,
            taxa::status,

            taxa::canonical_name,
            species::subspecies.nullable(),
            synonyms::names.nullable(),
            common_names::names.nullable(),
        ))
        .filter(taxa::status.eq_any(&[TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<UndescribedSpeciesDoc>(&mut conn)?;

    let docs = docs.into_iter().map(|doc| SpeciesDoc {
        name_id: doc.name_id,
        status: doc.status,
        canonical_name: doc.canonical_name,
        subspecies: doc.subspecies,
        synonyms: doc.synonyms,
        vernacular_names: doc.vernacular_names,
        kingdom: None,
        phylum: None,
        class: None,
        order: None,
        family: None,
        genus: None,
    }).collect();

    Ok(docs)
}
