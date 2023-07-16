mod taxon;

use tantivy::schema::Schema;
use tracing::info;

use tantivy::{doc, Index};

use anyhow::Error;
use crate::index::providers::search::SearchIndex;
use crate::database::Database;


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
    let synonyms = schema.get_field("synonyms").expect("synonyms schema field not found");
    let common_names = schema.get_field("common_names").expect("common_names schema field not found");

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
            if let Some(names) = &species.synonyms {
                for name in names {
                    doc.add_text(synonyms, name);
                }
            }
            if let Some(names) = &species.vernacular_names {
                for name in names {
                    doc.add_text(common_names, name);
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

    Ok(())
}
