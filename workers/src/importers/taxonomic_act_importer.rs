use std::path::PathBuf;

use arga_core::models::TaxonomicAct;
use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use tracing::info;

use crate::error::Error;
use crate::extractors::taxonomic_act_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let acts = taxonomic_act_extractor::extract(&path, pool)?;
    import_acts(&acts, pool)?;

    Ok(())
}

pub fn import_acts(records: &Vec<TaxonomicAct>, pool: &mut PgPool) -> Result<(), Error> {
    use diesel::upsert::excluded;
    use schema::taxonomic_acts::dsl::*;

    info!(total = records.len(), "Importing taxonomic acts");
    let imported: Vec<Result<usize, Error>> = records
        .par_chunks(1000)
        .map(|chunk| {
            let mut conn = pool.get()?;
            let inserted_rows = diesel::insert_into(taxonomic_acts)
                .values(chunk)
                .on_conflict(entity_id)
                .do_update()
                .set((
                    entity_id.eq(excluded(entity_id)),
                    taxon_id.eq(excluded(taxon_id)),
                    accepted_taxon_id.eq(excluded(accepted_taxon_id)),
                    act.eq(excluded(act)),
                    source_url.eq(excluded(source_url)),
                ))
                .execute(&mut conn)?;
            Ok(inserted_rows)
        })
        .collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total = records.len(), total_imported, "Importing taxanomic acts finished");

    Ok(())
}
