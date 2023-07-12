mod taxon;

use tantivy::schema::Schema;
use tracing::info;

use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

use tantivy::{doc, Index};
use uuid::Uuid;

use anyhow::Error;
use crate::index::providers::search::SearchIndex;
use crate::database::{schema, schema_gnl, Database};
use crate::database::models::{Name, CommonName};


#[derive(clap::Subcommand)]
pub enum Command {
    /// Create the search index
    Create,
    Reindex,
}

pub async fn process_command(command: &Command) {
    tracing_subscriber::fmt().init();

    match command {
        Command::Create => create().await.unwrap(),
        Command::Reindex => reindex().await.unwrap(),
    }
}


pub async fn create() -> Result<(), Error> {
    let schema = SearchIndex::schema()?;
    let index = Index::create_in_dir(".index", schema.clone())?;

    index_names(&schema, &index).await
}


pub async fn reindex() -> Result<(), Error> {
    let schema = SearchIndex::schema()?;
    let index = Index::open_in_dir(".index")?;

    {
        let mut index_writer = index.writer(500_000_000)?;
        index_writer.delete_all_documents()?;
        index_writer.commit()?;
    }

    index_names(&schema, &index).await
}


async fn index_names(schema: &Schema, index: &Index) -> Result<(), Error> {
    let db_host = crate::database::get_database_url();
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let name_id = schema.get_field("name_id").expect("name_id schema field not found");
    let canonical_name = schema.get_field("canonical_name").expect("canonical_name schema field not found");
    let subspecies = schema.get_field("subspecies").expect("subspecies schema field not found");
    // let common_names = schema.get_field("common_names").expect("common_names schema field not found");

    let genus = schema.get_field("genus").expect("genus schema field not found");
    let undescribed_species = schema.get_field("undescribed_species").expect("undescribed_species schema field not found");

    info!("Loading species from database");
    let species = taxon::get_species(&database).await?;

    for chunk in species.chunks(1_000_000) {
        for species in chunk {
            let mut doc = doc!(
                name_id => species.name_id.to_string(),
            );

            if let Some(name) = &species.canonical_name {
                doc.add_text(canonical_name, name);
            }

            if let Some(names) = &species.subspecies {
                for name in names {
                    doc.add_text(subspecies, name);
                }
            }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    info!("Loading undescribed species from database");
    let undescribed = taxon::get_undescribed_species(&database).await?;

    for chunk in undescribed.chunks(1_000_000) {
        for record in chunk {
            let mut doc = doc!(
                genus => record.genus.to_string(),
            );

            for name in &record.names {
                doc.add_text(undescribed_species, name);
            }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    // info!("Loading names from database");
    // let names = get_names(&database).await;

    // let mut vernacular_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    // let mut taxa_map: HashMap<Uuid, SpeciesDoc> = HashMap::new();

    // for chunk in names.chunks(50_000) {
    //     // collect all the vernacular names for each name id
    //     info!("Getting common names");
    //     let vernacular = get_vernacular_names(&database, &chunk).await;
    //     for name in vernacular {
    //         match vernacular_map.get_mut(&name.id) {
    //             Some(names) => { names.push(name.vernacular_name); }
    //             None => { vernacular_map.insert(name.id, vec![name.vernacular_name]); }
    //         };
    //     }

    //     // collect all the taxa details for each name id
    //     info!("Getting species");
    //     let taxa = taxon::get_species(&database, &chunk).await?;
    //     for taxon in taxa {
    //         if !taxa_map.contains_key(&taxon.name_id) {
    //             taxa_map.insert(taxon.name_id.clone(), taxon);
    //         }
    //     }

    //     info!(length=chunk.len(), "Indexing chunk");
    //     for name in chunk {
    //         let mut doc = doc!(
    //             scientific_name => name.scientific_name.clone(),
    //             name_id => name.id.to_string(),
    //         );

    //         if let Some(name) = &name.canonical_name {
    //             doc.add_text(canonical_name, name);
    //         }

    //         if let Some(names) = vernacular_map.get(&name.id) {
    //             for common in names {
    //                 doc.add_text(common_names, common);
    //             }
    //         }

    //         // if let Some(taxon) = taxa_map.get(&name.id) {
    //         //     doc.add_i64(taxa_priority, taxon.taxa_priority.into());
    //         //     if let Some(status) = &taxon.taxonomic_status {
    //         //         doc.add_text(taxonomic_status, status);
    //         //     }
    //         // }

    //         index_writer.add_document(doc)?;
    //     }
    //     index_writer.commit()?;
    // }

    Ok(())
}

async fn get_names(db: &Database) -> Vec<Name> {
    let mut conn = db.pool.get().await.unwrap();
    use schema::names::dsl::*;

    names
        .order_by(scientific_name)
        .load::<Name>(&mut conn)
        .await
        .unwrap()
}

async fn get_vernacular_names(db: &Database, names: &[Name]) -> Vec<CommonName> {
    let mut conn = db.pool.get().await.unwrap();
    use schema_gnl::common_names::dsl::*;

    let name_ids: Vec<&Uuid> = names.iter().map(|name| &name.id).collect();

    // only get names not assigned to a language for now
    common_names
        .filter(id.eq_any(&name_ids))
        .filter(vernacular_language.is_null())
        .order_by((scientific_name, vernacular_name))
        .load::<CommonName>(&mut conn)
        .await
        .unwrap()
}
