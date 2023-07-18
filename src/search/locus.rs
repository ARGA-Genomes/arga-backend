use serde::Deserialize;
use serde::Serialize;

use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

use uuid::Uuid;

use crate::database::models::TaxonomicStatus;
use crate::database::{schema, Database};
use crate::http::Error;


#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct LocusDoc {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub canonical_name: Option<String>,

    pub accession: String,
    pub locus_type: Option<String>,
}

pub async fn get_loci(db: &Database) -> Result<Vec<LocusDoc>, Error> {
    use schema::{markers, names, taxa};
    let mut conn = db.pool.get().await.unwrap();

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
        .load::<LocusDoc>(&mut conn)
        .await?;

    Ok(docs)
}
