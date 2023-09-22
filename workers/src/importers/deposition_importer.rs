use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{DepositionEvent, Dataset};
use crate::error::Error;
use crate::extractors::deposition_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting deposition events");

    let extractor = deposition_extractor::extract(path, &dataset, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_deposition_events(extract.deposition_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}

fn import_deposition_events(depositions: Vec<DepositionEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::deposition_events;

    info!(total=depositions.len(), "Importing deposition events");
    let imported: Vec<Result<usize, Error>> = depositions.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(deposition_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=depositions.len(), total_imported, "Importing deposition events finished");

    Ok(())
}
