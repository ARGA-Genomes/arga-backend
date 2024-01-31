use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use tracing::info;

use arga_core::schema;
use arga_core::models::TaxonPhoto;
use crate::error::Error;
use crate::extractors::taxon_photo_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let photos = taxon_photo_extractor::extract(&path, pool)?;
    import_photos(&photos, pool)?;

    Ok(())
}


fn import_photos(photos: &Vec<TaxonPhoto>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxon_photos;

    info!(total=photos.len(), "Importing taxon photos");
    let imported: Vec<Result<usize, Error>> = photos.chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(taxon_photos::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=photos.len(), total_imported, "Importing taxon photos finished");

    Ok(())
}
