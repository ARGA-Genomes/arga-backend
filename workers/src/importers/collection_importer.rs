use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{Specimen, CollectionEvent, Dataset};
use crate::error::Error;
use crate::extractors::collection_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting collection events");

    let extractor = collection_extractor::extract(path, &dataset, pool)?;

    for extract in extractor {
        let extract = extract?;
        // the extractors generate UUIDs and associate all records in the extract
        // so we must import them in a specific order to not trigger referential integrity
        // errors in the database.
        // right now we don't want to cross polinate datasets when it comes to linking
        // specimens or events to other specimens so this approach works for us as that
        // means every collection import should always create new records
        import_specimens(extract.specimens, pool)?;
        import_collection_events(extract.collection_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}


fn import_specimens(specimens: Vec<Specimen>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::specimens;

    info!(total=specimens.len(), "Importing specimens");
    let imported: Vec<Result<usize, Error>> = specimens.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(specimens::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=specimens.len(), total_imported, "Importing specimens finished");

    Ok(())
}

fn import_collection_events(collections: Vec<CollectionEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::collection_events;

    info!(total=collections.len(), "Importing specimen collection events");
    let imported: Vec<Result<usize, Error>> = collections.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(collection_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=collections.len(), total_imported, "Importing specimen collection events finished");

    Ok(())
}
