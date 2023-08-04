use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use crate::database::schema;
use crate::database::models::{TaxonSource, Name, Taxon};
use crate::workers::error::Error;
use crate::workers::extractors::{name_extractor, taxon_extractor};


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn get_or_create_dataset(
    name: &str,
    description: &Option<String>,
    url: &Option<String>,
    pool: &mut PgPool,
) -> Result<TaxonSource, Error>
{
    use schema::taxon_source;
    let mut conn = pool.get()?;

    if let Some(source) = taxon_source::table.filter(taxon_source::name.eq(name)).get_result(&mut conn).optional()? {
        return Ok(source);
    }

    let source = diesel::insert_into(taxon_source::table)
        .values((
            taxon_source::name.eq(name),
            taxon_source::description.eq(description),
            taxon_source::url.eq(url),
        ))
        .get_result(&mut conn)?;

    Ok(source)
}


pub fn import(path: PathBuf, source: &TaxonSource, pool: &mut PgPool) -> Result<(), Error> {
    // we always want to extract and import the names completely first because
    // other extractors rely on using the matcher to retreive the associated name id
    let names = name_extractor::extract(path.clone())?;
    import_names(&names, pool)?;

    let taxa = taxon_extractor::extract(path, source, pool)?;
    import_taxa(&taxa, pool)?;

    Ok(())
}


fn import_names(records: &Vec<Name>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::names;

    info!(total=records.len(), "Importing names");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=records.len(), total_imported, "Importing names finished");

    Ok(())
}


fn import_taxa(records: &Vec<Taxon>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxa;

    info!(total=records.len(), "Importing taxa");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(taxa::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=records.len(), total_imported, "Importing taxa finished");

    Ok(())
}
