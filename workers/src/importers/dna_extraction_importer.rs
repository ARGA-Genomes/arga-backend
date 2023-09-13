use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{Event, Dataset, DnaExtractionEvent};
use crate::error::Error;
use crate::extractors::dna_extraction_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn get_dataset(global_id: &str, pool: &mut PgPool) -> Result<Dataset, Error> {
    use schema::datasets;
    let mut conn = pool.get()?;
    let dataset = datasets::table.filter(datasets::global_id.eq(global_id)).get_result::<Dataset>(&mut conn)?;
    Ok(dataset)
}


pub fn import(path: PathBuf, global_id: &str, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting dna extraction events");

    let dataset = get_dataset(global_id, pool)?;
    let extractor = dna_extraction_extractor::extract(path, &dataset, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_events(extract.events, pool)?;
        import_dna_extraction_events(extract.dna_extraction_events, pool)?;
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

fn import_dna_extraction_events(extractions: Vec<DnaExtractionEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::dna_extraction_events;

    info!(total=extractions.len(), "Importing dna extraction events");
    let imported: Vec<Result<usize, Error>> = extractions.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(dna_extraction_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=extractions.len(), total_imported, "Importing dna extractions events finished");

    Ok(())
}
