mod taxon;
mod genome;
mod locus;

use diesel::*;
use diesel::r2d2::{ConnectionManager, Pool};

use tantivy::schema::{Schema, Field};
use tracing::info;

use tantivy::{doc, Index, DateTime};

use anyhow::Error;
use arga_core::search::{SearchIndex, DataType};


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Create the search index
    Create,
    Reindex,
}

pub fn process_command(command: &Command) {
    tracing_subscriber::fmt().init();

    match command {
        Command::Create => create().unwrap(),
        Command::Reindex => reindex().unwrap(),
    }
}


pub fn create() -> Result<(), Error> {
    let schema = SearchIndex::schema()?;
    let index = Index::create_in_dir(".index", schema.clone())?;

    index_names(&schema, &index)?;
    index_genomes(&schema, &index)?;
    index_loci(&schema, &index)?;
    Ok(())
}


pub fn reindex() -> Result<(), Error> {
    let schema = SearchIndex::schema()?;
    let index = Index::open_in_dir(".index")?;

    {
        let mut index_writer = index.writer(500_000_000)?;
        index_writer.delete_all_documents()?;
        index_writer.commit()?;
    }

    index_names(&schema, &index)?;
    index_genomes(&schema, &index)?;
    index_loci(&schema, &index)?;
    Ok(())
}


fn index_names(schema: &Schema, index: &Index) -> Result<(), Error> {
    let pool = get_pool()?;

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let data_type = get_field(schema, "data_type")?;
    let name_id = get_field(schema, "name_id")?;
    let status = get_field(schema, "status")?;
    let canonical_name = get_field(schema, "canonical_name")?;

    let subspecies = get_field(schema, "subspecies")?;
    let synonyms = get_field(schema, "synonyms")?;
    let common_names = get_field(schema, "common_names")?;

    let kingdom = get_field(schema, "kingdom")?;
    let phylum = get_field(schema, "phylum")?;
    let class = get_field(schema, "class")?;
    let order = get_field(schema, "order")?;
    let family = get_field(schema, "family")?;
    let genus = get_field(schema, "genus")?;

    info!("Loading species from database");
    let mut species = taxon::get_species(&pool)?;

    info!("Loading undescribed species from database");
    let undescribed = taxon::get_undescribed_species(&pool)?;
    species.extend(undescribed);

    for chunk in species.chunks(1_000_000) {
        for species in chunk {
            let mut doc = doc!(
                data_type => DataType::Taxon.to_string(),
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

    Ok(())
}

fn index_genomes(schema: &Schema, index: &Index) -> Result<(), Error> {
    let pool = get_pool()?;

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let data_type = get_field(schema, "data_type")?;
    let name_id = get_field(schema, "name_id")?;
    let status = get_field(schema, "status")?;
    let canonical_name = get_field(schema, "canonical_name")?;

    let accession = get_field(schema, "accession")?;
    let genome_rep = get_field(schema, "genome_rep")?;
    let level = get_field(schema, "level")?;
    let reference_genome = get_field(schema, "reference_genome")?;
    let release_date = get_field(schema, "release_date")?;

    info!("Loading assemblies from database");
    let records = genome::get_genomes(&pool)?;

    for chunk in records.chunks(1_000_000) {
        for genome in chunk {
            let mut doc = doc!(
                data_type => DataType::Genome.to_string(),
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


fn index_loci(schema: &Schema, index: &Index) -> Result<(), Error> {
    let pool = get_pool()?;

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let data_type = get_field(schema, "data_type")?;
    let name_id = get_field(schema, "name_id")?;
    let status = get_field(schema, "status")?;
    let canonical_name = get_field(schema, "canonical_name")?;

    let accession = get_field(schema, "accession")?;
    let locus_type = get_field(schema, "locus_type")?;

    info!("Loading loci from database");
    let records = locus::get_loci(&pool)?;

    for chunk in records.chunks(1_000_000) {
        for locus in chunk {
            let mut doc = doc!(
                data_type => DataType::Locus.to_string(),
                name_id => locus.name_id.to_string(),
                status => serde_json::to_string(&locus.status)?,
                accession => locus.accession.clone(),
            );

            if let Some(value) = &locus.canonical_name { doc.add_text(canonical_name, value); }
            if let Some(value) = &locus.locus_type { doc.add_text(locus_type, value); }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    Ok(())
}


fn get_field(schema: &Schema, name: &str) -> Result<Field, Error> {
    let field = schema.get_field(name).ok_or(tantivy::TantivyError::FieldNotFound(name.to_string()))?;
    Ok(field)
}


fn get_pool() -> Result<PgPool, Error> {
    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}
