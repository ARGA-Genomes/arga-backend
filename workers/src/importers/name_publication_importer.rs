use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{NamePublication, Dataset};
use crate::error::Error;
use crate::extractors::name_publication_extractor;

type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    let publications = name_publication_extractor::extract(&path, dataset)?;
    import_name_publications(&publications, pool)?;

    Ok(())
}


fn import_name_publications(publications: &Vec<NamePublication>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::name_publications;

    info!(total=publications.len(), "Importing name publications");
    let imported: Vec<Result<usize, Error>> = publications.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(name_publications::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=publications.len(), total_imported, "Importing name publications finished");

    Ok(())
}
