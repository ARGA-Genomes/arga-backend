use std::path::PathBuf;

use arga_core::models::NomenclaturalAct;
use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use tracing::info;

use crate::error::Error;
use crate::extractors::nomenclatural_act_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let acts = nomenclatural_act_extractor::extract(&path, pool)?;
    import_nomenclatural_acts(&acts, pool)?;

    Ok(())
}

fn import_nomenclatural_acts(acts: &Vec<NomenclaturalAct>, pool: &mut PgPool) -> Result<(), Error> {
    use diesel::upsert::excluded;
    use schema::nomenclatural_acts::dsl::*;

    info!(total = acts.len(), "Importing nomenclatural acts");
    let imported: Vec<Result<usize, Error>> = acts
        .par_chunks(1000)
        .map(|chunk| {
            let mut conn = pool.get()?;
            let inserted_rows = diesel::insert_into(nomenclatural_acts)
                .values(chunk)
                .on_conflict(name_id)
                .do_update()
                .set((
                    entity_id.eq(excluded(entity_id)),
                    publication_id.eq(excluded(publication_id)),
                    name_id.eq(excluded(name_id)),
                    acted_on_id.eq(excluded(acted_on_id)),
                    act.eq(excluded(act)),
                    source_url.eq(excluded(source_url)),
                    created_at.eq(excluded(created_at)),
                    updated_at.eq(excluded(updated_at)),
                ))
                .execute(&mut conn)?;
            Ok(inserted_rows)
        })
        .collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total = acts.len(), total_imported, "Importing nomenclatural acts finished");

    Ok(())
}
