use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{Name, Dataset};
use crate::error::Error;
use crate::extractors::name_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    let names = name_extractor::extract(&path)?;
    import_names(&names, pool)?;

    Ok(())
}


pub fn import_names(records: &Vec<Name>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::names;

    info!(total=records.len(), "Importing names");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=records.len(), total_imported, "Importing names finished");

    Ok(())
}
