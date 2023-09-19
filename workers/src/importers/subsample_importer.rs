use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{Event, Dataset, SubsampleEvent, Subsample};
use crate::error::Error;
use crate::extractors::subsample_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting subsample events");

    let extractor = subsample_extractor::extract(path, &dataset, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_events(extract.events, pool)?;
        import_subsamples(extract.subsamples, pool)?;
        import_subsample_events(extract.subsample_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}


fn import_events(events: Vec<Event>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::events;

    info!(total=events.len(), "Importing events");
    let imported: Vec<Result<usize, Error>> = events.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=events.len(), imported=total_imported, "Importing events finished");

    Ok(())
}

fn import_subsamples(subsamples: Vec<Subsample>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::subsamples;

    info!(total=subsamples.len(), "Importing subsamples");
    let imported: Vec<Result<usize, Error>> = subsamples.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(subsamples::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=subsamples.len(), total_imported, "Importing subsamples finished");

    Ok(())
}

fn import_subsample_events(subsamples: Vec<SubsampleEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::subsample_events;

    info!(total=subsamples.len(), "Importing subsample events");
    let imported: Vec<Result<usize, Error>> = subsamples.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(subsample_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=subsamples.len(), total_imported, "Importing subsample events finished");

    Ok(())
}
