use anyhow::Error;
use arga_core::{schema, schema_gnl};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{RunQueryDsl, *};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct LocusDoc {
    pub name_id: Uuid,
    // pub status: TaxonomicStatus,
    pub canonical_name: String,

    pub accession: String,
    pub data_source: String,
    pub locus_type: String,
    pub event_date: Option<String>,
}

pub fn get_loci(pool: &PgPool) -> Result<Vec<LocusDoc>, Error> {
    use schema::names;
    use schema_gnl::markers;
    let mut conn = pool.get()?;

    let docs = markers::table
        .inner_join(names::table)
        // .inner_join(taxa::table.on(names::id.eq(taxa::name_id)))
        .select((
            names::id,
            names::canonical_name,
            // taxa::name_id,
            // taxa::status,
            // taxa::canonical_name,
            markers::record_id,
            markers::dataset_name,
            markers::target_gene,
            markers::release_date,
        ))
        // .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<LocusDoc>(&mut conn)?;

    Ok(docs)
}
