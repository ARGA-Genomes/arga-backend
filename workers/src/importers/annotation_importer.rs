use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{Event, AnnotationEvent, Dataset};
use crate::error::Error;
use crate::extractors::annotation_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting annotation events");

    let extractor = annotation_extractor::extract(path, &dataset, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_events(extract.events, pool)?;
        import_annotation_events(extract.annotation_events, pool)?;
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

fn import_annotation_events(annotations: Vec<AnnotationEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::annotation_events;

    info!(total=annotations.len(), "Importing annotation events");
    let imported: Vec<Result<usize, Error>> = annotations.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(annotation_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=annotations.len(), total_imported, "Importing annotation events finished");

    Ok(())
}
