use anyhow::Error;
use arga_core::models::{ACCEPTED_NAMES, TaxonomicStatus};
use arga_core::schema::datasets;
use arga_core::schema_gnl;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{Nullable, Text, Varchar};
use diesel::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


pub const ALA_DATASET_ID: &str = "ARGA:TL:0001013";

type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct SpeciesDoc {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,

    pub canonical_name: String,
    pub rank: String,
    // pub subspecies: Option<Vec<String>>,
    // pub synonyms: Option<Vec<String>>,
    pub vernacular_names: Option<Vec<String>>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,

    pub regnum: Option<String>,
    pub division: Option<String>,
    pub classis: Option<String>,
    pub ordo: Option<String>,
    pub familia: Option<String>,
}

pub fn get_species(pool: &PgPool) -> Result<Vec<SpeciesDoc>, Error> {
    use diesel::dsl::sql;
    use schema_gnl::species;
    // use schema_gnl::{species, synonyms, common_names};
    let mut conn = pool.get()?;

    let docs = species::table
        .inner_join(datasets::table.on(species::dataset_id.eq(datasets::id)))
        // .left_join(synonyms::table)
        .select((
            species::id,
            species::status,
            species::canonical_name,
            sql::<Text>("species.rank::text"),
            // species::subspecies,
            // synonyms::names.nullable(),
            species::vernacular_names,
            sql::<Nullable<Varchar>>("classification->>'kingdom'"),
            sql::<Nullable<Varchar>>("classification->>'phylum'"),
            sql::<Nullable<Varchar>>("classification->>'class'"),
            sql::<Nullable<Varchar>>("classification->>'order'"),
            sql::<Nullable<Varchar>>("classification->>'family'"),
            sql::<Nullable<Varchar>>("classification->>'genus'"),
            sql::<Nullable<Varchar>>("classification->>'regnum'"),
            sql::<Nullable<Varchar>>("classification->>'division'"),
            sql::<Nullable<Varchar>>("classification->>'classis'"),
            sql::<Nullable<Varchar>>("classification->>'ordo'"),
            sql::<Nullable<Varchar>>("classification->>'familia'"),
        ))
        .filter(species::status.eq_any(&ACCEPTED_NAMES))
        .filter(datasets::global_id.eq(ALA_DATASET_ID))
        .load::<SpeciesDoc>(&mut conn)?;

    Ok(docs)
}


// #[derive(Debug, Queryable, Serialize, Deserialize)]
// pub struct UndescribedSpeciesDoc {
//     pub name_id: Uuid,
//     pub status: TaxonomicStatus,

//     pub canonical_name: String,
//     pub subspecies: Option<Vec<String>>,
//     pub synonyms: Option<Vec<String>>,
//     pub vernacular_names: Option<Vec<String>>,
// }

// pub fn get_undescribed_species(pool: &PgPool) -> Result<Vec<SpeciesDoc>, Error> {
//     use schema::taxa;
//     use schema_gnl::{species, synonyms, common_names};
//     let mut conn = pool.get()?;

//     let docs = taxa::table
//         .left_join(species::table)
//         .left_join(synonyms::table)
//         .left_join(common_names::table)
//         .select((
//             taxa::name_id,
//             taxa::status,

//             taxa::canonical_name,
//             species::subspecies.nullable(),
//             synonyms::names.nullable(),
//             common_names::names.nullable(),
//         ))
//         .filter(taxa::status.eq_any(&[TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
//         .load::<UndescribedSpeciesDoc>(&mut conn)?;

//     let docs = docs.into_iter().map(|doc| SpeciesDoc {
//         name_id: doc.name_id,
//         status: doc.status,
//         canonical_name: doc.canonical_name,
//         subspecies: doc.subspecies,
//         synonyms: doc.synonyms,
//         vernacular_names: doc.vernacular_names,
//         kingdom: None,
//         phylum: None,
//         class: None,
//         order: None,
//         family: None,
//         genus: None,
//     }).collect();

//     Ok(docs)
// }
