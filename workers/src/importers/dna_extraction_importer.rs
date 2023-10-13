use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{Dataset, DnaExtractionEvent, DnaExtract};
use crate::error::Error;
use crate::extractors::dna_extraction_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, context: &Vec<Dataset>, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting dna extraction events");

    let extractor = dna_extraction_extractor::extract(path, dataset, context, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_dna_extracts(extract.dna_extracts, pool)?;
        import_dna_extraction_events(extract.dna_extraction_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}

fn import_dna_extracts(extracts: Vec<DnaExtract>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::dna_extracts;

    info!(total=extracts.len(), "Importing dna extracts");
    let imported: Vec<Result<usize, Error>> = extracts.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(dna_extracts::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=extracts.len(), total_imported, "Importing dna extracts finished");

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
