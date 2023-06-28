use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;
use tantivy::schema::Schema;
use tracing::info;

use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

use tantivy::{doc, Index};
use uuid::Uuid;

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

    index_names(&schema, &index).await
}


pub async fn reindex() -> tantivy::Result<()> {
    let schema = SearchIndex::schema()?;
    let index = Index::open_in_dir(".index")?;

    {
        let mut index_writer = index.writer(500_000_000)?;
        index_writer.delete_all_documents()?;
        index_writer.commit()?;
    }

    index_names(&schema, &index).await
}


async fn index_names(schema: &Schema, index: &Index) -> tantivy::Result<()> {
    let db_host = std::env::var("DATABASE_URL").expect("No database url specified");
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let name_id = schema.get_field("name_id").expect("name_id schema field not found");
    let scientific_name = schema.get_field("scientific_name").expect("scientific_name schema field not found");
    let canonical_name = schema.get_field("canonical_name").expect("canonical_name schema field not found");
    let common_names = schema.get_field("common_names").expect("common_names schema field not found");
    let rank = schema.get_field("rank").expect("rank schema field not found");
    let taxonomic_status = schema.get_field("taxonomic_status").expect("taxonomic_status schema field not found");
    let taxa_priority = schema.get_field("name_id").expect("name_id schema field not found");


    info!("Loading names from database");
    let names = get_names(&database).await;

    let mut vernacular_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    let mut taxa_map: HashMap<Uuid, Taxon> = HashMap::new();

    for chunk in names.chunks(50_000) {
        // collect all the vernacular names for each name id
        info!("Getting common names");
        let vernacular = get_vernacular_names(&database, &chunk).await;
        for name in vernacular {
            match vernacular_map.get_mut(&name.id) {
                Some(names) => { names.push(name.vernacular_name); }
                None => { vernacular_map.insert(name.id, vec![name.vernacular_name]); }
            };
        }

        // collect all the taxa details for each name id
        info!("Getting taxa");
        let taxa = get_taxa(&database, &chunk).await;
        for taxon in taxa {
            if !taxa_map.contains_key(&taxon.name_id) {
                taxa_map.insert(taxon.name_id.clone(), taxon);
            }
        }

        info!(length=chunk.len(), "Indexing chunk");
        for name in chunk {
            let mut doc = doc!(
                scientific_name => name.scientific_name.clone(),
                canonical_name => name.canonical_name.clone().unwrap_or_default(),
                rank => name.rank.clone(),
                name_id => name.id.to_string(),
            );

            if let Some(names) = vernacular_map.get(&name.id) {
                for common in names {
                    doc.add_text(common_names, common);
                }
            }

            if let Some(taxon) = taxa_map.get(&name.id) {
                doc.add_i64(taxa_priority, taxon.taxa_priority.into());
                if let Some(status) = &taxon.taxonomic_status {
                    doc.add_text(taxonomic_status, status);
                }
            }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

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


#[derive(Debug, Queryable, Serialize, Deserialize)]
struct Taxon {
    pub name_id: Uuid,
    pub taxonomic_status: Option<String>,
    pub taxa_priority: i32,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
}

async fn get_taxa(db: &Database, names: &[Name]) -> Vec<Taxon> {
    let mut conn = db.pool.get().await.unwrap();
    use schema::{user_taxa, user_taxa_lists};

    let name_ids: Vec<&Uuid> = names.iter().map(|name| &name.id).collect();

    // get taxa details with the list details for boosting
    user_taxa::table
        .inner_join(user_taxa_lists::table)
        .select((
            user_taxa::name_id,
            user_taxa::taxonomic_status,
            user_taxa_lists::priority,
            user_taxa::kingdom,
            user_taxa::phylum,
            user_taxa::class,
            user_taxa::order,
            user_taxa::family,
            user_taxa::genus,
        ))
        .filter(user_taxa::name_id.eq_any(&name_ids))
        .order(user_taxa_lists::priority)
        .load::<Taxon>(&mut conn)
        .await
        .unwrap()
}
