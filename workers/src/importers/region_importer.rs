use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{Regions, NameList, NameListType};
use crate::error::Error;
use crate::extractors::region_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn get_or_create_dataset(list_name: &str, list_description: &Option<String>, pool: &mut PgPool) -> Result<NameList, Error> {
    use schema::name_lists;
    let mut conn = pool.get()?;

    if let Some(list) = name_lists::table.filter(name_lists::name.eq(list_name)).get_result(&mut conn).optional()? {
        return Ok(list);
    }

    let list = diesel::insert_into(name_lists::table)
        .values((
            name_lists::list_type.eq(NameListType::Regions),
            name_lists::name.eq(list_name),
            name_lists::description.eq(list_description),
        ))
        .get_result(&mut conn)?;

    Ok(list)
}


pub fn import(path: PathBuf, list: &NameList, pool: &mut PgPool) -> Result<(), Error> {
    let regions = region_extractor::extract(path, list, pool)?;
    import_regions(&regions, pool)?;
    Ok(())
}


fn import_regions(regions: &Vec<Regions>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::regions;

    info!(total=regions.len(), "Importing regions");
    let imported: Vec<Result<usize, Error>> = regions.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(regions::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=regions.len(), total_imported, "Importing regions finished");

    Ok(())
}
