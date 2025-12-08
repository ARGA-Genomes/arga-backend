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
    pub canonical_name: String,

    pub accession: Option<String>,
    pub institution_code: Option<String>,
    pub collection_repository_id: Option<String>,
    pub collection_repository_code: Option<String>,
    pub collected_by: Option<String>,
    pub identified_by: Option<String>,
    pub event_date: Option<chrono::NaiveDate>,
}

pub fn get_specimen_total(pool: &PgPool) -> Result<u64, Error> {
    use schema::{collection_events, names, specimens};
    let mut conn = pool.get()?;

    let total = specimens::table
        .inner_join(collection_events::table)
        .inner_join(names::table)
        .count()
        .get_result::<i64>(&mut conn)?;

    Ok(total as u64)
}

pub fn get_specimens(pool: &PgPool, page: i64, page_size: i64) -> Result<Vec<SpecimenDoc>, Error> {
    use schema::{accession_events, collection_events, names, specimens};
    let mut conn = pool.get()?;

    let docs = specimens::table
        .inner_join(collection_events::table)
        .inner_join(names::table)
        .inner_join(accession_events::table)
        .select((
            names::id,
            names::canonical_name,
            specimens::specimen_id,
            accession_events::institution_code,
            accession_events::collection_repository_id,
            accession_events::collection_repository_code,
            collection_events::collected_by,
            accession_events::identified_by,
            collection_events::event_date,
        ))
        .offset((page - 1) * page_size)
        .limit(page_size)
        .load::<SpecimenDoc>(&mut conn)?;

    Ok(docs)
}
