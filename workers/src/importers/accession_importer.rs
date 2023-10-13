use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{AccessionEvent, Dataset};
use crate::error::Error;
use crate::extractors::accession_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, context: &Vec<Dataset>, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting accession events");

    let extractor = accession_extractor::extract(path, dataset, context, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_accession_events(extract.accession_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}

fn import_accession_events(accessions: Vec<AccessionEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::accession_events;

    info!(total=accessions.len(), "Importing accession events");
    let imported: Vec<Result<usize, Error>> = accessions.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(accession_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=accessions.len(), total_imported, "Importing accession events finished");

    Ok(())
}
