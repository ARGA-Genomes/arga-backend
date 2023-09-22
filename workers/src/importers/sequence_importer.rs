use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{SequencingEvent, Dataset, Sequence};
use crate::error::Error;
use crate::extractors::sequence_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, context: &Vec<Dataset>, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting sequencing events");

    let extractor = sequence_extractor::extract(path, &dataset, context, pool)?;

    for extract in extractor {
        let extract = extract?;
        import_sequences(extract.sequences, pool)?;
        import_sequencing_events(extract.sequencing_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}

fn import_sequences(sequences: Vec<Sequence>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::sequences;

    info!(total=sequences.len(), "Importing sequences");
    let imported: Vec<Result<usize, Error>> = sequences.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(sequences::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=sequences.len(), imported=total_imported, "Importing sequnces finished");

    Ok(())
}

fn import_sequencing_events(sequences: Vec<SequencingEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::sequencing_events;

    info!(total=sequences.len(), "Importing sequencing events");
    let imported: Vec<Result<usize, Error>> = sequences.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(sequencing_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=sequences.len(), total_imported, "Importing sequencing events finished");

    Ok(())
}
