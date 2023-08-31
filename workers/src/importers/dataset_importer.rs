use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::Dataset;
use crate::error::Error;
use crate::extractors::dataset_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let records = dataset_extractor::extract(path, pool)?;
    import_datasets(&records, pool)?;
    Ok(())
}


fn import_datasets(records: &Vec<Dataset>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::datasets;

    info!(total=records.len(), "Importing datasets");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(datasets::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=records.len(), total_imported, "Importing datasets finished");

    Ok(())
}
