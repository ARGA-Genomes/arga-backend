use serde::Deserialize;
use serde::Serialize;

use diesel::*;
use diesel::RunQueryDsl;
use diesel::r2d2::{ConnectionManager, Pool};

use uuid::Uuid;
use anyhow::Error;

use arga_core::models::TaxonomicStatus;
use arga_core::schema;


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct LocusDoc {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub canonical_name: Option<String>,

    pub accession: String,
    pub locus_type: Option<String>,
}

pub fn get_loci(pool: &PgPool) -> Result<Vec<LocusDoc>, Error> {
    use schema::{markers, names, taxa};
    let mut conn = pool.get()?;

    let docs = names::table
        .inner_join(taxa::table)
        .inner_join(markers::table)
        .select((
            taxa::name_id,
            taxa::status,
            taxa::canonical_name,
            markers::accession,
            markers::type_,
        ))
        .filter(taxa::status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<LocusDoc>(&mut conn)?;

    Ok(docs)
}
