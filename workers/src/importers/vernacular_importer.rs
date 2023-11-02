use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use crate::error::Error;
use crate::extractors::vernacular_extractor::{self, VernacularName, VernacularNameLink};


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let vernacular_names = vernacular_extractor::extract(&path)?;
    import_vernacular(&vernacular_names, pool)?;

    let vernacular_links = vernacular_extractor::extract_links(&path, pool)?;
    import_vernacular_links(&vernacular_links, pool)?;

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


fn import_vernacular_links(names: &Vec<VernacularNameLink>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::name_vernacular_names;

    info!(total=names.len(), "Importing vernacular name links");
    let imported: Vec<Result<usize, Error>> = names.chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(name_vernacular_names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=names.len(), total_imported, "Importing vernacular name links finished");

    Ok(())
}
