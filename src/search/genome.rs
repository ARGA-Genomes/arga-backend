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
pub struct GenomeDoc {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub canonical_name: Option<String>,

    pub accession: String,
    pub genome_rep: Option<String>,
    pub level: Option<String>,
    pub reference_genome: Option<String>,
    pub release_date: Option<String>,
}

pub async fn get_genomes(db: &Database) -> Result<Vec<GenomeDoc>, Error> {
    use schema::{assemblies, names, taxa};
    let mut conn = db.pool.get().await.unwrap();

    let docs = names::table
        .inner_join(taxa::table)
        .inner_join(assemblies::table)
        .select((
            taxa::name_id,
            taxa::status,
            taxa::canonical_name,
            assemblies::accession,
            assemblies::genome_rep,
            assemblies::contam_screen_input,
            assemblies::refseq_category,
            assemblies::event_date,
        ))
        .filter(taxa::status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Hybrid, TaxonomicStatus::Undescribed]))
        .load::<GenomeDoc>(&mut conn)
        .await?;

    Ok(docs)
}
