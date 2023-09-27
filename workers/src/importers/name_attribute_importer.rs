use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{NameAttribute, Dataset};
use crate::error::Error;
use crate::extractors::name_attribute_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let attrs = name_attribute_extractor::extract(path, pool)?;
    import_name_attributes(&attrs, pool)?;
    Ok(())
}


fn import_name_attributes(attrs: &Vec<NameAttribute>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::name_attributes;

    info!(total=attrs.len(), "Importing name attributes");
    let imported: Vec<Result<usize, Error>> = attrs.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(name_attributes::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=attrs.len(), total_imported, "Importing name attributes finished");

    Ok(())
}
