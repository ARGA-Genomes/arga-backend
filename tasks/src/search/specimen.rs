use anyhow::Error;
use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{RunQueryDsl, *};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct SpecimenDoc {
    pub name_id: Uuid,
    // pub status: TaxonomicStatus,
    pub canonical_name: String,

    pub accession: String,
    pub data_source: String,
    pub institution_code: Option<String>,
    pub collection_code: Option<String>,
    pub recorded_by: Option<String>,
    pub identified_by: Option<String>,
    pub event_date: Option<String>,
}

pub fn get_specimens(pool: &PgPool) -> Result<Vec<SpecimenDoc>, Error> {
    use schema::{collection_events, datasets, names, specimens};
    let mut conn = pool.get()?;

    let docs = specimens::table
        .inner_join(datasets::table)
        .inner_join(collection_events::table)
        .inner_join(names::table)
        // .inner_join(taxa::table.on(taxa::name_id.eq(names::id)))
        .select((
            names::id,
            names::canonical_name,
            // taxa::name_id,
            // taxa::status,
            // taxa::canonical_name,
            specimens::record_id,
            datasets::name,
            specimens::institution_code,
            specimens::collection_code,
            specimens::recorded_by,
            specimens::identified_by,
            collection_events::event_date,
        ))
        // .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<SpecimenDoc>(&mut conn)?;

    Ok(docs)
}
