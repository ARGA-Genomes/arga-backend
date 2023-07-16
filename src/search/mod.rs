mod taxon;
mod genome;

use tantivy::schema::Schema;
use tracing::info;

use tantivy::{doc, Index, DateTime};

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

    index_names(&schema, &index).await?;
    index_genomes(&schema, &index).await?;
    Ok(())
}


pub async fn reindex() -> Result<(), Error> {
    let schema = SearchIndex::schema()?;
    let index = Index::open_in_dir(".index")?;

    {
        let mut index_writer = index.writer(500_000_000)?;
        index_writer.delete_all_documents()?;
        index_writer.commit()?;
    }

    index_names(&schema, &index).await?;
    index_genomes(&schema, &index).await?;
    Ok(())
}


async fn index_names(schema: &Schema, index: &Index) -> Result<(), Error> {
    let db_host = crate::database::get_database_url();
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let data_type = schema.get_field("data_type").expect("data_type schema field not found");
    let name_id = schema.get_field("name_id").expect("name_id schema field not found");
    let status = schema.get_field("status").expect("status schema field not found");

    let canonical_name = schema.get_field("canonical_name").expect("canonical_name schema field not found");
    let subspecies = schema.get_field("subspecies").expect("subspecies schema field not found");
    let synonyms = schema.get_field("synonyms").expect("synonyms schema field not found");
    let common_names = schema.get_field("common_names").expect("common_names schema field not found");

    let kingdom = schema.get_field("kingdom").expect("kingdom schema field not found");
    let phylum = schema.get_field("phylum").expect("phylum schema field not found");
    let class = schema.get_field("class").expect("class schema field not found");
    let order = schema.get_field("order").expect("order schema field not found");
    let family = schema.get_field("family").expect("family schema field not found");
    let genus = schema.get_field("genus").expect("genus schema field not found");

    info!("Loading species from database");
    let species = taxon::get_species(&database).await?;

    for chunk in species.chunks(1_000_000) {
        for species in chunk {
            let mut doc = doc!(
                data_type => "taxon",
                name_id => species.name_id.to_string(),
                status => serde_json::to_string(&species.status)?,
            );

            if let Some(name) = &species.canonical_name { doc.add_text(canonical_name, name); }
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

            if let Some(value) = &species.kingdom { doc.add_text(kingdom, value); }
            if let Some(value) = &species.phylum { doc.add_text(phylum, value); }
            if let Some(value) = &species.class { doc.add_text(class, value); }
            if let Some(value) = &species.order { doc.add_text(order, value); }
            if let Some(value) = &species.family { doc.add_text(family, value); }
            if let Some(value) = &species.genus { doc.add_text(genus, value); }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    // info!("Loading undescribed species from database");
    // let undescribed = taxon::get_undescribed_species(&database).await?;

    // for chunk in undescribed.chunks(1_000_000) {
    //     for record in chunk {
    //         let mut doc = doc!(
    //             genus => record.genus.to_string(),
    //         );

    //         for name in &record.names {
    //             doc.add_text(undescribed_species, name);
    //         }

    //         index_writer.add_document(doc)?;
    //     }
    //     index_writer.commit()?;
    // }

    Ok(())
}

async fn index_genomes(schema: &Schema, index: &Index) -> Result<(), Error> {
    let db_host = crate::database::get_database_url();
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let data_type = schema.get_field("data_type").expect("data_type schema field not found");
    let name_id = schema.get_field("name_id").expect("name_id schema field not found");
    let status = schema.get_field("status").expect("status schema field not found");
    let canonical_name = schema.get_field("canonical_name").expect("canonical_name schema field not found");

    let accession = schema.get_field("accession").expect("accession schema field not found");
    let genome_rep = schema.get_field("genome_rep").expect("genome_rep schema field not found");
    let level = schema.get_field("level").expect("level schema field not found");
    let reference_genome = schema.get_field("reference_genome").expect("reference_genome schema field not found");
    let release_date = schema.get_field("release_date").expect("release_date schema field not found");

    info!("Loading assemblies from database");
    let records = genome::get_genomes(&database).await?;

    for chunk in records.chunks(1_000_000) {
        for genome in chunk {
            let mut doc = doc!(
                data_type => "genome",
                name_id => genome.name_id.to_string(),
                status => serde_json::to_string(&genome.status)?,
                accession => genome.accession.clone(),
            );

            if let Some(value) = &genome.canonical_name { doc.add_text(canonical_name, value); }
            if let Some(value) = &genome.genome_rep { doc.add_text(genome_rep, value); }
            if let Some(value) = &genome.level { doc.add_text(level, value); }
            if let Some(value) = &genome.reference_genome {
                doc.add_bool(reference_genome, value == "reference genome");
            }
            if let Some(value) = &genome.release_date {
                if let Ok(date) = chrono::NaiveDateTime::parse_from_str(value, "%Y/%m/%d") {
                    let timestamp = DateTime::from_timestamp_secs(date.timestamp());
                    doc.add_date(release_date, timestamp);
                }
            }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    Ok(())
}
