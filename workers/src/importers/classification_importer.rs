use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::NewClassification;
use crate::error::Error;
use crate::extractors::classification_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let classifications = classification_extractor::extract(&path, pool)?;
    import_classifications(&classifications, pool)?;

    Ok(())
}

pub fn import_classifications(records: &Vec<NewClassification>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::classifications;

    info!(total=records.len(), "Importing classifications");
    let mut conn = pool.get()?;

    let total_imported = conn.transaction::<usize, Error, _>(|conn| {
        let mut total_imported = 0;

        for chunk in records.chunks(1000) {
            let inserted_rows = diesel::insert_into(classifications::table)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(conn)?;
            total_imported += inserted_rows;
        }

        Ok(total_imported)
    })?;

    // let imported: Vec<Result<usize, Error>> = records.par_chunks(1000).map(|chunk| {
    //     let mut conn = pool.get()?;
    //     let inserted_rows = diesel::insert_into(classifications::table)
    //         .values(chunk)
    //         .on_conflict_do_nothing()
    //         .execute(&mut conn)?;
    //     Ok(inserted_rows)
    // }).collect();

    // let mut total_imported = 0;
    // for chunk_total in imported {
    //     total_imported += chunk_total?;
    // }
    info!(total=records.len(), total_imported, "Importing classifications finished");

    Ok(())
}
