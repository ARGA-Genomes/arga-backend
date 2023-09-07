use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::Ecology;
use crate::error::Error;
use crate::extractors::ecology_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let records = ecology_extractor::extract(path, pool)?;
    import_ecology(&records, pool)?;
    Ok(())
}


fn import_ecology(records: &Vec<Ecology>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::ecology;

    info!(total=records.len(), "Importing ecology");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(ecology::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=records.len(), total_imported, "Importing ecology finished");

    Ok(())
}
