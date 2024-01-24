use std::path::PathBuf;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::{info, error};

use arga_core::schema;
use arga_core::models::{Taxon, TaxonName};
use crate::error::Error;
use crate::extractors::classification_extractor;
use crate::matchers::name_matcher::name_map;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, pool: &mut PgPool) -> Result<(), Error> {
    let classifications = classification_extractor::extract(&path, pool)?;
    import_classifications(&classifications, pool)?;
    link_classifications(&classifications, pool)?;
    link_names(&classifications, pool)?;

    Ok(())
}

pub fn import_classifications(records: &Vec<Taxon>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxa;

    info!(total=records.len(), "Importing classifications");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(1000).map(|chunk| {
        let mut node_chunk = Vec::new();
        for record in chunk {
            let mut node = record.clone();
            node.parent_id = None;
            node_chunk.push(node);
        }

        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(taxa::table)
            .values(node_chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }

    info!(total=records.len(), total_imported, "Importing classifications finished");
    Ok(())
}


pub fn link_classifications(records: &Vec<Taxon>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxa::dsl::*;

    info!(total=records.len(), "Linking classifications");
    let imported: Vec<Result<usize, Error>> = records.par_chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let mut linked = 0;

        for record in chunk {
            let result = diesel::update(taxa.filter(id.eq(record.id)))
                .set(parent_id.eq(record.parent_id))
                .get_result::<Taxon>(&mut conn)?;

            if result.parent_id.is_some() {
                linked += 1;
            }
        }

        Ok(linked)
    }).collect();

    let mut total_linked = 0;
    for chunk_total in imported {
        total_linked += chunk_total?;
    }

    info!(total=records.len(), total_linked, "Linking classifications finished");
    Ok(())
}


pub fn link_names(records: &Vec<Taxon>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxa_names::dsl::*;

    info!(total=records.len(), "Linking classifications");
    let names = name_map(pool)?;

    let imported: Vec<Result<usize, Error>> = records.par_chunks(10_000).map(|chunk| {
        let mut links = Vec::new();
        let mut conn = pool.get()?;

        for record in chunk {
            match names.get(&record.scientific_name) {
                Some(name_match) => {
                    links.push(TaxonName {
                        taxon_id: record.id,
                        name_id: name_match.id,
                    });
                },
                None => {
                    error!(id=?record.id, name=record.scientific_name, "Failed to find a matching name");
                }
            }
        }

        let inserted_rows = diesel::insert_into(taxa_names)
            .values(links)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;

        Ok(inserted_rows)
    }).collect();

    let mut total_linked = 0;
    for chunk_total in imported {
        total_linked += chunk_total?;
    }

    info!(total=records.len(), total_linked, "Linking classifications finished");
    Ok(())
}
