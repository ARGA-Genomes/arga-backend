use std::collections::HashMap;
use tracing::info;

use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

use tantivy::{doc, Index};

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


pub async fn create() -> tantivy::Result<()> {
    let schema = SearchIndex::schema()?;
    let index = Index::create_in_dir(".index", schema.clone())?;

    let db_host = std::env::var("DATABASE_URL").expect("No database url specified");
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;
    let scientific_name = schema.get_field("scientific_name").expect("scientific_name schema field not found");
    let canonical_name = schema.get_field("canonical_name").expect("canonical_name schema field not found");
    let rank = schema.get_field("rank").expect("rank schema field not found");
    let common_names = schema.get_field("common_names").expect("common_names schema field not found");


    info!("Loading names from database");
    let names = get_names(&database).await;

    let mut vernacular_map: HashMap<String, Vec<String>> = HashMap::new();

    for chunk in names.chunks(1_000_000) {
        info!("Getting common names");
        let vernacular = get_vernacular_names(&database, &chunk).await;

        for name in vernacular {
            match vernacular_map.get_mut(&name.scientific_name) {
                Some(names) => {
                    names.push(name.vernacular_name);
                },
                None => {
                    vernacular_map.insert(name.scientific_name, vec![name.vernacular_name]);
                },
            };
        }

        info!(length=chunk.len(), "Indexing chunk");
        for name in chunk {
            let mut doc = doc!(
                scientific_name => name.scientific_name.clone(),
                canonical_name => name.canonical_name.clone().unwrap_or_default(),
                rank => name.rank.clone(),
            );

            if let Some(names) = vernacular_map.get(&name.scientific_name) {
                for common in names {
                    doc.add_text(common_names, common);
                }
            }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    Ok(())
}


pub async fn reindex() -> tantivy::Result<()> {
    let schema = SearchIndex::schema()?;
    let index = Index::create_in_dir(".index", schema.clone())?;

    let mut index_writer = index.writer(500_000_000)?;
    index_writer.delete_all_documents()?;
    index_writer.commit()?;

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

    let names: Vec<&String> = names.iter().map(|name| &name.scientific_name).collect();

    // only get names not assigned to a language for now
    common_names
        .filter(scientific_name.eq_any(&names))
        .filter(vernacular_language.is_null())
        .order_by((vernacular_name, scientific_name))
        .load::<CommonName>(&mut conn)
        .await
        .unwrap()
}
