use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;

use arga_core::schema;
use arga_core::models::{TaxonomicStatus, TaxonHistory};
use crate::error::Error;
use crate::extractors::{name_extractor, taxon_extractor, taxon_history_extractor};

use super::taxon_importer::{import_taxa, import_names};


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    // synonyms are a superset of a taxonomy and taxon history, so we import the synonym
    // name and taxonomy before building the history
    let names = name_extractor::extract(&path)?;
    import_names(&names, pool)?;

    // after extracting the taxa we make sure that all of them have a taxonomic status
    // of synonym since we are explicitly importing synonyms here
    let mut taxa = taxon_extractor::extract(&path, pool)?;
    for taxon in taxa.iter_mut() {
        taxon.status = TaxonomicStatus::Synonym;
    }
    import_taxa(&taxa, pool)?;

    let history = taxon_history_extractor::extract(&path, pool)?;
    import_taxa_history(&history, pool)?;

    Ok(())
}


fn import_taxa_history(history: &Vec<TaxonHistory>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxon_history;

    info!(total=history.len(), "Importing taxa history");
    let imported: Vec<Result<usize, Error>> = history.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(taxon_history::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=history.len(), total_imported, "Importing taxa history finished");

    Ok(())
}
