use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::AdminMedia;
use crate::error::Error;
use crate::extractors::admin_media_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, source: String, pool: &mut PgPool) -> Result<(), Error> {
    let media = admin_media_extractor::extract(path, &source, pool)?;
    import_admin_media(&media, pool)?;
    Ok(())
}


fn import_admin_media(media: &Vec<AdminMedia>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::admin_media;

    info!(total=media.len(), "Importing admin media");
    let imported: Vec<Result<usize, Error>> = media.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(admin_media::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=media.len(), total_imported, "Importing admin media finished");

    Ok(())
}
