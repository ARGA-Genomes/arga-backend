use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{IndigenousKnowledge, Dataset};
use crate::error::Error;
use crate::extractors::indigenous_knowledge_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn get_or_create_dataset(name: &str, description: &Option<String>, pool: &mut PgPool) -> Result<Dataset, Error> {
    use schema::datasets;
    let mut conn = pool.get()?;

    if let Some(dataset) = datasets::table.filter(datasets::name.eq(name)).get_result(&mut conn).optional()? {
        return Ok(dataset);
    }

    let list = diesel::insert_into(datasets::table)
        .values((
            datasets::name.eq(name),
            datasets::description.eq(description),
        ))
        .get_result(&mut conn)?;

    Ok(list)
}


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    let records = indigenous_knowledge_extractor::extract(path, dataset, pool)?;
    import_indigenous_knowledge(&records, pool)?;
    Ok(())
}


fn import_indigenous_knowledge(records: &Vec<IndigenousKnowledge>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::indigenous_knowledge;

    info!(total=records.len(), "Importing indigenous knowledge");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(indigenous_knowledge::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=records.len(), total_imported, "Importing indigenous knowledge finished");

    Ok(())
}
