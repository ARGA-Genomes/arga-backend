use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{AssemblyEvent, Dataset};
use crate::error::Error;
use crate::extractors::assembly_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting assembly events");

    let extractor = assembly_extractor::extract(path, &dataset, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_assembly_events(extract.assembly_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}

fn import_assembly_events(assemblies: Vec<AssemblyEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::assembly_events;

    info!(total=assemblies.len(), "Importing assembly events");
    let imported: Vec<Result<usize, Error>> = assemblies.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(assembly_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=assemblies.len(), total_imported, "Importing assembly events finished");

    Ok(())
}
