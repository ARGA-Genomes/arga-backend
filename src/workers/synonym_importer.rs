use std::path::{Path, PathBuf};
use std::collections::HashMap;

use chrono::Utc;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::database::schema;
use crate::database::models::{Job, Name, TaxonSource, Taxon, TaxonomicStatus, TaxonHistory};


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub struct SynonymImporter {
    thread: PipedThread<Job, Job>,
}

impl SynonymImporter {
    pub fn init(cx: CX![]) -> Option<Self> {
        let thread = PipedThread::spawn(
            fwd_to!([cx], recv() as (Job)),
            fwd_to!([cx], term() as (Option<String>)),
            cx,
            move |link| {
                while let Some(job) = link.recv() {
                    Self::process(job);
                }
            }
        );

        Some(Self {
            thread,
        })
    }

    pub fn import(&mut self, _cx: CX![], job: Job) {
        self.thread.send(job);
    }

    fn recv(&mut self, _cx: CX![], _job: Job) {

    }

    fn term(&self, cx: CX![], panic: Option<String>) {
        if let Some(msg) = panic {
            panic!("Unexpected thread failure: {}", msg);
        }
        cx.stop();
    }

    // #[instrument]
    fn process(job: Job) {
        info!("Running synonym importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let source = get_or_create_taxon_source(&data.name, &data.description, &data.url).unwrap();
                    let path = Path::new(&tmp_path).join(data.tmp_name);
                    import(path, &source).unwrap();
                }
                Err(err) => {
                    error!(?err, "Invalid JSON payload");
                }
            }
        }
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error occurred with the database connection")]
    Database(#[from] diesel::result::Error),

    #[error("an error occurred parsing the file")]
    Csv(#[from] csv::Error),

    #[error("an error occurred getting a database connection")]
    Pool(#[from] diesel::r2d2::PoolError),
}

#[derive(Debug, Deserialize)]
struct ImportJobData {
    name: String,
    description: Option<String>,
    url: Option<String>,
    tmp_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    valid_scientific_name: String,

    scientific_name: String,
    // authority: Option<String>,
    canonical_name: Option<String>,
    // rank: Option<String>,

    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    tribe: Option<String>,
    genus: Option<String>,

    superclass: Option<String>,
    superorder: Option<String>,
    superfamily: Option<String>,
    supertribe: Option<String>,

    subphylum: Option<String>,
    subclass: Option<String>,
    suborder: Option<String>,
    subfamily: Option<String>,
    subtribe: Option<String>,
    subgenus: Option<String>,
    // subspecies: Option<String>,

    specific_epithet: Option<String>,
    subspecific_epithet: Option<String>,

    species: Option<String>,
    genus_full: Option<String>,
    family_full: Option<String>,
    order_full: Option<String>,

    // name_according_to: Option<String>,
    // name_published_in: Option<String>,

    // taxon_remarks: Option<String>,
    // source: Option<String>,
    // source_url: Option<String>,
    // source_id: Option<String>,

    change_reason: Option<String>,
}

#[derive(Debug, Queryable, Deserialize)]
struct NameMatch {
    id: Uuid,
    scientific_name: String,
}

#[derive(Debug, Queryable, Deserialize)]
struct TaxonMatch {
    id: Uuid,
    scientific_name: String,
}


pub fn get_or_create_taxon_source(name: &str, description: &Option<String>, url: &Option<String>) -> Result<TaxonSource, Error> {
    use schema::taxon_source;
    let pool = get_connection_pool();
    let mut conn = pool.get()?;

    if let Some(source) = taxon_source::table.filter(taxon_source::name.eq(name)).get_result(&mut conn).optional()? {
        return Ok(source);
    }

    let source = diesel::insert_into(taxon_source::table)
        .values((
            taxon_source::name.eq(name),
            taxon_source::description.eq(description),
            taxon_source::url.eq(url),
        ))
        .get_result(&mut conn)?;

    Ok(source)
}


#[instrument]
pub fn import(path: PathBuf, source: &TaxonSource) -> Result<(), Error> {
    info!("Getting database connection pool");
    let pool = &mut get_connection_pool();

    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    import_names(&records, pool)?;
    import_synonyms(&records, source, pool)?;
    import_taxa_history(&records, source, pool)?;

    Ok(())
}


fn import_names(records: &Vec<Record>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::names;

    let names = extract_names(&records);

    info!(total=records.len(), "Importing names");
    let imported: Vec<Result<usize, Error>> = names.par_chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=names.len(), total_imported, "Importing names finished");

    Ok(())
}


fn import_synonyms(records: &Vec<Record>, source: &TaxonSource, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxa;

    let names = match_names(&records, pool);
    let synonyms = extract_synonyms(&source, &names, &records);

    // filter out unmatched specimens
    let synonyms = synonyms.into_iter().filter_map(|r| r).collect::<Vec<Taxon>>();

    info!(total=synonyms.len(), "Importing synonyms");
    let imported: Vec<Result<usize, Error>> = synonyms.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(taxa::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=synonyms.len(), total_imported, "Importing synonyms finished");

    Ok(())
}


fn import_taxa_history(records: &Vec<Record>, source: &TaxonSource, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxon_history;

    let taxa = match_taxa(&records, pool);
    let history = extract_taxa_history(&source, &taxa, &records);

    // filter out unmatched taxa history
    let history = history.into_iter().filter_map(|r| r).collect::<Vec<TaxonHistory>>();

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


fn match_names(records: &Vec<Record>, pool: &mut PgPool) -> HashMap<String, Uuid> {
    use schema::names;
    info!(total=records.len(), "Matching names");

    let matched: Vec<Result<Vec<NameMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let scientific_names: Vec<&String> = chunk.iter().map(|row| &row.scientific_name).collect();

        let results = names::table
            .select((names::id, names::scientific_name))
            .filter(names::scientific_name.eq_any(scientific_names))
            .load::<NameMatch>(&mut conn)?;

        Ok::<Vec<NameMatch>, Error>(results)
    }).collect();

    let mut id_map: HashMap<String, Uuid> = HashMap::new();

    for chunk in matched {
        if let Ok(names) = chunk {
            for name_match in names {
                id_map.insert(name_match.scientific_name, name_match.id);
            }
        }
    }

    info!(total=records.len(), matched=id_map.len(), "Matching names finished");
    id_map
}


fn match_taxa(records: &Vec<Record>, pool: &mut PgPool) -> HashMap<String, Uuid> {
    use schema::taxa;
    info!(total=records.len(), "Matching taxa");

    let matched: Vec<Result<Vec<TaxonMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;

        let mut names: Vec<&String> = chunk.iter().map(|row| &row.scientific_name).collect();
        let valid_names: Vec<&String> = chunk.iter().map(|row| &row.valid_scientific_name).collect();
        names.extend(valid_names);

        let results = taxa::table
            .select((taxa::id, taxa::scientific_name))
            .filter(taxa::scientific_name.eq_any(names))
            .load::<TaxonMatch>(&mut conn)?;

        Ok::<Vec<TaxonMatch>, Error>(results)
    }).collect();

    let mut id_map: HashMap<String, Uuid> = HashMap::new();

    for chunk in matched {
        if let Ok(names) = chunk {
            for taxon_match in names {
                id_map.insert(taxon_match.scientific_name, taxon_match.id);
            }
        }
    }

    info!(total=records.len(), matched=id_map.len(), "Matching names finished");
    id_map
}


fn extract_names(records: &Vec<Record>) -> Vec<Name> {
    info!(total=records.len(), "Extracting names");

    let names = records.par_iter().map(|row| {
        let species_authority = extract_authority(&row.canonical_name, &row.species);

        Name {
            id: Uuid::new_v4(),
            scientific_name: row.scientific_name.clone(),
            canonical_name: row.canonical_name.clone(),
            authorship: species_authority,
        }
    }).collect::<Vec<Name>>();

    info!(names=names.len(), "Extracting names finished");
    names
}


fn extract_synonyms(source: &TaxonSource, names: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<Taxon>> {
    info!(total=records.len(), "Extracting synonyms");

    let taxa = records.par_iter().map(|row| {
        let order_authority = extract_authority(&row.order, &row.order_full);
        let family_authority = extract_authority(&row.family, &row.family_full);
        let genus_authority = extract_authority(&row.genus, &row.genus_full);
        let species_authority = extract_authority(&row.canonical_name, &row.species);

        match names.get(&row.scientific_name) {
            Some(name_id) => Some(Taxon {
                id: Uuid::new_v4(),
                source: source.id.clone(),
                name_id: name_id.clone(),

                status: TaxonomicStatus::Synonym,
                scientific_name: row.scientific_name.clone(),
                canonical_name: row.canonical_name.clone(),

                kingdom: row.kingdom.clone(),
                phylum: row.phylum.clone(),
                class: row.class.clone(),
                order: row.order.clone(),
                family: row.family.clone(),
                tribe: row.tribe.clone(),
                genus: row.genus.clone(),
                specific_epithet: row.specific_epithet.clone(),

                subphylum: row.subphylum.clone(),
                subclass: row.subclass.clone(),
                suborder: row.suborder.clone(),
                subfamily: row.subfamily.clone(),
                subtribe: row.subtribe.clone(),
                subgenus: row.subgenus.clone(),
                subspecific_epithet: row.subspecific_epithet.clone(),

                superclass: row.superclass.clone(),
                superorder: row.superorder.clone(),
                superfamily: row.superfamily.clone(),
                supertribe: row.supertribe.clone(),

                order_authority,
                family_authority,
                genus_authority,
                species_authority,
            }),
            None => None,
        }
    }).collect::<Vec<Option<Taxon>>>();

    info!(taxa=taxa.len(), "Extracting synonyms finished");
    taxa
}


fn extract_taxa_history(source: &TaxonSource, taxa: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<TaxonHistory>> {
    info!(total=records.len(), "Extracting taxa history");

    let history = records.par_iter().map(|row| {
        let old_taxon_id = taxa.get(&row.scientific_name);
        let new_taxon_id = taxa.get(&row.valid_scientific_name);
        let changed_by = format!("Import: {}", source.name);

        match (old_taxon_id, new_taxon_id) {
            (Some(old_taxon_id), Some(new_taxon_id)) => Some(TaxonHistory {
                id: Uuid::new_v4(),
                old_taxon_id: old_taxon_id.clone(),
                new_taxon_id: new_taxon_id.clone(),
                changed_by: Some(changed_by),
                reason: row.change_reason.clone(),
                created_at: Utc::now(),
            }),
            _ => None,
        }
    }).collect::<Vec<Option<TaxonHistory>>>();

    info!(history=history.len(), "Extracting taxa history finished");
    history
}


fn extract_authority(name: &Option<String>, full_name: &Option<String>) -> Option<String> {
    match (name, full_name) {
        (Some(name), Some(full_name)) => Some(full_name.trim_start_matches(name).trim().to_string()),
        _ => None
    }
}


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = crate::database::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder().build(manager).expect("Could not build connection pool")
}
