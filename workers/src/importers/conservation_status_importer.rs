use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{ConservationStatus, Dataset};
use crate::error::Error;
use crate::extractors::conservation_status_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    let statuses = conservation_status_extractor::extract(path, &dataset, pool)?;
    import_conservation_status(&statuses, pool)?;
    Ok(())
}


fn import_conservation_status(statuses: &Vec<ConservationStatus>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::conservation_statuses;

    info!(total=statuses.len(), "Importing conservation status");
    let imported: Vec<Result<usize, Error>> = statuses.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(conservation_statuses::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=statuses.len(), total_imported, "Importing conservation status finished");

    Ok(())
}
