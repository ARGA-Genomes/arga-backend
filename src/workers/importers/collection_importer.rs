use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use crate::database::schema;
use crate::database::models::{NameList, NameListType, Specimen, Event, CollectionEvent, Organism};
use crate::workers::error::Error;
use crate::workers::extractors::collection_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn create_dataset(list_name: &str, list_description: &Option<String>, pool: &mut PgPool) -> Result<NameList, Error> {
    use schema::name_lists::dsl::*;
    let mut conn = pool.get()?;

    let source = diesel::insert_into(name_lists)
        .values((
            list_type.eq(NameListType::Specimen),
            name.eq(list_name),
            description.eq(list_description),
        ))
        .get_result(&mut conn)?;

    Ok(source)
}

pub fn import(path: PathBuf, list: &NameList, pool: &mut PgPool) -> Result<(), Error> {
    let extract = collection_extractor::extract(path, list, pool)?;

    // the extractors generate UUIDs and associate all records in the extract
    // so we must import them in a specific order to not trigger referential integrity
    // errors in the database.
    // right now we don't want to cross polinate datasets when it comes to linking
    // specimens or events to other specimens so this approach works for us as that
    // means every collection import should always create new records
    import_specimens(extract.specimens, pool)?;
    import_organisms(extract.organisms, pool)?;
    import_events(extract.events, pool)?;
    import_collection_events(extract.collection_events, pool)?;

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

fn import_organisms(organisms: Vec<Organism>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::organisms;

    info!(total=organisms.len(), "Importing specimen organisms");
    let imported: Vec<Result<usize, Error>> = organisms.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(organisms::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=organisms.len(), imported=total_imported, "Importing specimen organisms finished");

    Ok(())
}

fn import_events(events: Vec<Event>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::events;

    info!(total=events.len(), "Importing specimen events");
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
    info!(total=events.len(), imported=total_imported, "Importing specimen events finished");

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
