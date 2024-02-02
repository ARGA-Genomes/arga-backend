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
pub struct GenomeDoc {
    pub name_id: Uuid,
    // pub status: TaxonomicStatus,
    pub canonical_name: String,

    pub data_source: String,
    pub accession: Option<String>,
    pub genome_rep: Option<String>,
    pub level: Option<String>,
    pub assembly_type: Option<String>,
    pub release_date: Option<String>,
}

pub fn get_genomes(pool: &PgPool) -> Result<Vec<GenomeDoc>, Error> {
    use schema_gnl::whole_genomes;
    use schema::{names, taxa};
    let mut conn = pool.get()?;

    let docs = whole_genomes::table
        .inner_join(names::table.on(whole_genomes::name_id.eq(names::id)))
        // .inner_join(taxa::table.on(names::id.eq(taxa::name_id)))
        .select((
            names::id,
            names::canonical_name,
            // taxa::name_id,
            // taxa::status,
            // taxa::canonical_name,
            whole_genomes::dataset_name,
            whole_genomes::accession,
            whole_genomes::representation,
            whole_genomes::quality,
            whole_genomes::assembly_type,
            whole_genomes::release_date,
        ))
        // .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<GenomeDoc>(&mut conn)?;

    Ok(docs)
}
