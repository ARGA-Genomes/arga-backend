mod genome;
mod locus;
mod specimen;
mod taxon;

use anyhow::Error;
use arga_core::search::{DataType, SearchIndex};
use chrono::NaiveTime;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use tantivy::schema::{Field, Schema};
use tantivy::{DateTime, Index, doc};
use tracing::info;


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
    index_specimens(&schema, &index)?;
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
    index_specimens(&schema, &index)?;
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
    let rank = get_field(schema, "rank")?;

    // let subspecies = get_field(schema, "subspecies")?;
    // let synonyms = get_field(schema, "synonyms")?;
    let common_names = get_field(schema, "common_names")?;

    let kingdom = get_field(schema, "kingdom")?;
    let phylum = get_field(schema, "phylum")?;
    let class = get_field(schema, "class")?;
    let order = get_field(schema, "order")?;
    let family = get_field(schema, "family")?;
    let genus = get_field(schema, "genus")?;

    let regnum = get_field(schema, "regnum")?;
    let division = get_field(schema, "division")?;
    let classis = get_field(schema, "classis")?;
    let ordo = get_field(schema, "ordo")?;
    let familia = get_field(schema, "familia")?;

    info!("Loading species from database");
    let species = taxon::get_species(&pool)?;

    // info!("Loading undescribed species from database");
    // let undescribed = taxon::get_undescribed_species(&pool)?;
    // species.extend(undescribed);
    info!(total = species.len(), "Loaded");

    for chunk in species.chunks(1_000_000) {
        for species in chunk {
            let mut doc = doc!(
                canonical_name => species.canonical_name.clone(),
                rank => species.rank.clone(),
                data_type => DataType::Taxon.to_string(),
                name_id => species.name_id.to_string(),
                status => serde_json::to_string(&species.status)?,
            );

            // if let Some(names) = &species.subspecies {
            //     for name in names {
            //         doc.add_text(subspecies, name);
            //     }
            // }
            // if let Some(names) = &species.synonyms {
            //     for name in names {
            //         doc.add_text(synonyms, name);
            //     }
            // }
            if let Some(names) = &species.vernacular_names {
                for name in names {
                    doc.add_text(common_names, name);
                }
            }

            if let Some(value) = &species.kingdom {
                doc.add_text(kingdom, value);
            }
            if let Some(value) = &species.phylum {
                doc.add_text(phylum, value);
            }
            if let Some(value) = &species.class {
                doc.add_text(class, value);
            }
            if let Some(value) = &species.order {
                doc.add_text(order, value);
            }
            if let Some(value) = &species.family {
                doc.add_text(family, value);
            }
            if let Some(value) = &species.genus {
                doc.add_text(genus, value);
            }

            if let Some(value) = &species.regnum {
                doc.add_text(regnum, value);
            }
            if let Some(value) = &species.division {
                doc.add_text(division, value);
            }
            if let Some(value) = &species.classis {
                doc.add_text(classis, value);
            }
            if let Some(value) = &species.ordo {
                doc.add_text(ordo, value);
            }
            if let Some(value) = &species.familia {
                doc.add_text(familia, value);
            }

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
    // let status = get_field(schema, "status")?;
    let canonical_name = get_field(schema, "canonical_name")?;

    let accession = get_field(schema, "accession")?;
    let data_source = get_field(schema, "data_source")?;
    let genome_rep = get_field(schema, "genome_rep")?;
    let level = get_field(schema, "level")?;
    let assembly_type = get_field(schema, "assembly_type")?;
    let release_date = get_field(schema, "release_date")?;
    let source_uri = get_field(schema, "source_uri")?;

    info!("Loading assemblies from database");
    let records = genome::get_genomes(&pool)?;
    info!(total = records.len(), "Loaded");

    for chunk in records.chunks(1_000_000) {
        for genome in chunk {
            let mut doc = doc!(
                canonical_name => genome.canonical_name.clone(),
                data_type => DataType::Genome.to_string(),
                name_id => genome.name_id.to_string(),
                // status => serde_json::to_string(&genome.status)?,
                data_source => genome.data_source.clone(),
            );

            if let Some(value) = &genome.accession {
                doc.add_text(accession, value);
            }
            if let Some(value) = &genome.genome_rep {
                doc.add_text(genome_rep, value);
            }
            if let Some(value) = &genome.level {
                doc.add_text(level, value);
            }
            if let Some(value) = &genome.assembly_type {
                doc.add_text(assembly_type, value);
            }
            if let Some(value) = &genome.release_date {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(value, "%Y/%m/%d") {
                    let timestamp =
                        DateTime::from_timestamp_secs(date.and_time(NaiveTime::default()).and_utc().timestamp());
                    doc.add_date(release_date, timestamp);
                }
            }
            if let Some(value) = &genome.source_uri {
                doc.add_text(source_uri, value);
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
    // let status = get_field(schema, "status")?;
    let canonical_name = get_field(schema, "canonical_name")?;

    let accession = get_field(schema, "accession")?;
    let data_source = get_field(schema, "data_source")?;
    let locus_type = get_field(schema, "locus_type")?;
    let event_date = get_field(schema, "event_date")?;

    info!("Loading loci from database");
    let records = locus::get_loci(&pool)?;
    info!(total = records.len(), "Loaded");

    for chunk in records.chunks(1_000_000) {
        for locus in chunk {
            let mut doc = doc!(
                canonical_name => locus.canonical_name.clone(),
                data_type => DataType::Locus.to_string(),
                name_id => locus.name_id.to_string(),
                // status => serde_json::to_string(&locus.status)?,
                accession => locus.accession.clone(),
                data_source => locus.data_source.clone(),
                locus_type => locus.locus_type.clone(),
            );

            if let Some(value) = &locus.event_date {
                doc.add_text(event_date, value);
            }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    Ok(())
}


fn index_specimens(schema: &Schema, index: &Index) -> Result<(), Error> {
    let pool = get_pool()?;

    // index some data with 500mb memory heap
    let mut index_writer = index.writer(500_000_000)?;

    let data_type = get_field(schema, "data_type")?;
    let name_id = get_field(schema, "name_id")?;
    // let status = get_field(schema, "status")?;
    let canonical_name = get_field(schema, "canonical_name")?;

    let accession = get_field(schema, "accession")?;
    let data_source = get_field(schema, "data_source")?;
    let institution_code = get_field(schema, "institution_code")?;
    let collection_code = get_field(schema, "collection_code")?;
    let recorded_by = get_field(schema, "recorded_by")?;
    let identified_by = get_field(schema, "identified_by")?;
    let event_date = get_field(schema, "event_date")?;

    info!("Getting total amount of specimens");
    let page_size: u64 = 500_000;
    let total = specimen::get_specimen_total(&pool)?;
    let pages = total.div_ceil(page_size);

    info!(total, pages, "Loading specimens from database");

    for page in 1..pages {
        let records = specimen::get_specimens(&pool, page as i64, page_size as i64)?;
        info!(page, total = pages, "Loaded");

        for specimen in records {
            let mut doc = doc!(
                canonical_name => specimen.canonical_name.clone(),
                data_type => DataType::Specimen.to_string(),
                name_id => specimen.name_id.to_string(),
                // status => serde_json::to_string(&specimen.status)?,
                accession => specimen.accession.clone(),
                data_source => specimen.data_source.clone(),
            );

            if let Some(value) = &specimen.institution_code {
                doc.add_text(institution_code, value);
            }
            if let Some(value) = &specimen.collection_code {
                doc.add_text(collection_code, value);
            }
            if let Some(value) = &specimen.recorded_by {
                doc.add_text(recorded_by, value);
            }
            if let Some(value) = &specimen.identified_by {
                doc.add_text(identified_by, value);
            }
            if let Some(value) = &specimen.event_date {
                doc.add_text(event_date, value);
            }

            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
    }

    Ok(())
}


fn get_field(schema: &Schema, name: &str) -> Result<Field, Error> {
    let field = schema
        .get_field(name)
        .ok_or(tantivy::TantivyError::FieldNotFound(name.to_string()))?;
    Ok(field)
}


fn get_pool() -> Result<PgPool, Error> {
    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}
