use arga_core::schema_gnl;
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
    pub canonical_name: String,

    pub accession: String,
    pub locus_type: String,
}

pub fn get_loci(pool: &PgPool) -> Result<Vec<LocusDoc>, Error> {
    use schema::{names, taxa};
    use schema_gnl::markers;
    let mut conn = pool.get()?;

    let docs = names::table
        .inner_join(taxa::table)
        .inner_join(markers::table)
        .select((
            taxa::name_id,
            taxa::status,
            taxa::canonical_name,
            markers::accession,
            markers::target_gene,
        ))
        .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<LocusDoc>(&mut conn)?;

    Ok(docs)
}
