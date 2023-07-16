use std::path::{Path, PathBuf};
use std::collections::HashMap;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::database::schema;
use crate::database::models::Job;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub struct VernacularImporter {
    thread: PipedThread<Job, Job>,
}

impl VernacularImporter {
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

    fn process(job: Job) {
        info!("Running vernacular importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let path = Path::new(&tmp_path).join(data.tmp_name);
                    import(path).unwrap();
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
    // name: String,
    // description: Option<String>,
    // url: Option<String>,
    tmp_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    valid_scientific_name: String,
    vernacular_name: String,
}

#[derive(Debug, Queryable, Deserialize)]
struct NameMatch {
    id: Uuid,
    scientific_name: String,
}

#[derive(Debug, Queryable, Deserialize)]
struct VernacularMatch {
    id: i64,
    vernacular_name: String,
}

#[derive(Debug, Insertable, Queryable, Deserialize)]
#[diesel(table_name = schema::vernacular_names)]
struct VernacularName {
    vernacular_name: String,
    language: Option<String>,
}

#[derive(Debug, Insertable, Queryable, Deserialize)]
#[diesel(table_name = schema::name_vernacular_names)]
struct VernacularNameLink {
    name_id: Uuid,
    vernacular_name_id: i64,
}


#[instrument]
pub fn import(path: PathBuf) -> Result<(), Error> {
    info!("Getting database connection pool");
    let pool = &mut get_connection_pool();

    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    import_vernacular(&records, pool)?;
    import_vernacular_links(&records, pool)?;

    Ok(())
}


fn import_vernacular(records: &Vec<Record>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::vernacular_names;

    let names = match_names(&records, pool);
    let vernacular = extract_vernacular(&names, &records);

    // filter out unmatched names
    let vernacular = vernacular.into_iter().filter_map(|r| r).collect::<Vec<VernacularName>>();

    info!(total=vernacular.len(), "Importing vernacular names");
    let imported: Vec<Result<usize, Error>> = vernacular.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(vernacular_names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=vernacular.len(), total_imported, "Importing vernacular names finished");

    Ok(())
}

fn import_vernacular_links(records: &Vec<Record>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::name_vernacular_names;

    let names = match_names(&records, pool);
    let vernacular = match_vernacular_names(&records, pool);
    let links = extract_vernacular_links(&names, &vernacular, &records);

    // filter out unmatched names
    let links = links.into_iter().filter_map(|r| r).collect::<Vec<VernacularNameLink>>();

    info!(total=links.len(), "Importing vernacular name links");
    let imported: Vec<Result<usize, Error>> = links.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(name_vernacular_names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=links.len(), total_imported, "Importing vernacular name links finished");

    Ok(())
}


fn match_names(records: &Vec<Record>, pool: &mut PgPool) -> HashMap<String, Uuid> {
    use schema::names;
    info!(total=records.len(), "Matching names");

    let matched: Vec<Result<Vec<NameMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let scientific_names: Vec<&String> = chunk.iter().map(|row| &row.valid_scientific_name).collect();

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


fn match_vernacular_names(records: &Vec<Record>, pool: &mut PgPool) -> HashMap<String, i64> {
    use schema::vernacular_names;
    info!(total=records.len(), "Matching vernacular names");

    let matched: Vec<Result<Vec<VernacularMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let vernacular_names: Vec<&String> = chunk.iter().map(|row| &row.vernacular_name).collect();

        let results = vernacular_names::table
            .select((vernacular_names::id, vernacular_names::vernacular_name))
            .filter(vernacular_names::vernacular_name.eq_any(vernacular_names))
            .load::<VernacularMatch>(&mut conn)?;

        Ok::<Vec<VernacularMatch>, Error>(results)
    }).collect();

    let mut id_map: HashMap<String, i64> = HashMap::new();

    for chunk in matched {
        if let Ok(names) = chunk {
            for vernacular_match in names {
                id_map.insert(vernacular_match.vernacular_name, vernacular_match.id);
            }
        }
    }

    info!(total=records.len(), matched=id_map.len(), "Matching vernacular names finished");
    id_map
}


fn extract_vernacular(names: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<VernacularName>> {
    info!(total=records.len(), "Extracting vernacular names");

    let vernacular_names = records.par_iter().map(|row| {
        match names.get(&row.valid_scientific_name) {
            Some(_name_id) => Some(VernacularName {
                vernacular_name: row.vernacular_name.clone(),
                language: Some("en".to_string()),
            }),
            None => None,
        }
    }).collect::<Vec<Option<VernacularName>>>();

    info!(vernacular_names=vernacular_names.len(), "Extracting vernacular names finished");
    vernacular_names
}


fn extract_vernacular_links(
    names: &HashMap<String, Uuid>,
    vernacular: &HashMap<String, i64>,
    records: &Vec<Record>,
) -> Vec<Option<VernacularNameLink>>
{
    info!(total=records.len(), "Extracting vernacular name links");

    let links = records.par_iter().map(|row| {
        let scientific_name = names.get(&row.valid_scientific_name);
        let vernacular_name = vernacular.get(&row.vernacular_name);

        match (scientific_name, vernacular_name) {
            (Some(name_id), Some(vernacular_name_id)) => Some(VernacularNameLink {
                name_id: name_id.clone(),
                vernacular_name_id: *vernacular_name_id,
            }),
            _ => None,
        }
    }).collect::<Vec<Option<VernacularNameLink>>>();

    info!(links=links.len(), "Extracting vernacular name links finished");
    links
}


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = crate::database::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder().build(manager).expect("Could not build connection pool")
}
