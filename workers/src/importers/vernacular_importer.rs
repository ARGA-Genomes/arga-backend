use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use tracing::info;

use arga_core::schema;
use arga_core::models::{Dataset, VernacularName};
use crate::error::Error;
use crate::extractors::vernacular_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, pool: &mut PgPool) -> Result<(), Error> {
    let vernacular_names = vernacular_extractor::extract(&path, dataset, pool)?;
    import_vernacular(&vernacular_names, pool)?;

    Ok(())
}


fn import_vernacular(names: &Vec<VernacularName>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::vernacular_names;

    info!(total=names.len(), "Importing vernacular names");
    let imported: Vec<Result<usize, Error>> = names.chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(vernacular_names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=names.len(), total_imported, "Importing vernacular names finished");

    Ok(())
}
