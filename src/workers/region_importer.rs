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
use crate::database::models::{Job, Regions, RegionType, NameList, NameListType};


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub struct RegionImporter {
    thread: PipedThread<Job, Job>,
}

impl RegionImporter {
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
        info!("Running region importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let list = get_or_create_region_list(&data.name, &data.description).unwrap();
                    let path = Path::new(&tmp_path).join(data.tmp_name);
                    import(path, &list).unwrap();
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
    tmp_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    region_type: String,
    regions: String,
}

#[derive(Debug, Queryable, Deserialize)]
struct NameMatch {
    id: Uuid,
    scientific_name: String,
}


pub fn get_or_create_region_list(name: &str, description: &Option<String>) -> Result<NameList, Error> {
    use schema::name_lists;
    let pool = get_connection_pool();
    let mut conn = pool.get()?;

    if let Some(list) = name_lists::table.filter(name_lists::name.eq(name)).get_result(&mut conn).optional()? {
        return Ok(list);
    }

    let list = diesel::insert_into(name_lists::table)
        .values((
            name_lists::list_type.eq(NameListType::Regions),
            name_lists::name.eq(name),
            name_lists::description.eq(description),
        ))
        .get_result(&mut conn)?;

    Ok(list)
}

#[instrument]
pub fn import(path: PathBuf, list: &NameList) -> Result<(), Error> {
    info!("Getting database connection pool");
    let pool = &mut get_connection_pool();

    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    import_regions(&records, pool)?;

    Ok(())
}


fn import_regions(records: &Vec<Record>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::regions;

    let names = match_names(&records, pool);
    let regions = extract_regions(&names, &records);

    // filter out unmatched names
    let regions = regions.into_iter().filter_map(|r| r).collect::<Vec<Regions>>();

    info!(total=regions.len(), "Importing regions");
    let imported: Vec<Result<usize, Error>> = regions.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(regions::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=regions.len(), total_imported, "Importing regions finished");

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


fn extract_regions(names: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<Regions>> {
    info!(total=records.len(), "Extracting regions");

    let regions = records.par_iter().map(|row| {
        let region_type = extract_region_type(&row.region_type);
        let name_id = names.get(&row.scientific_name);

        match (name_id, region_type) {
            (Some(name_id), Some(region_type)) => Some(Regions {
                id: Uuid::new_v4(),
                name_id: name_id.clone(),
                values: extract_region_values(&row.regions),
                region_type,
            }),
            _ => None,
        }
    }).collect::<Vec<Option<Regions>>>();

    info!(regions=regions.len(), "Extracting regions finished");
    regions
}


fn extract_region_type(region_type: &str) -> Option<RegionType> {
    match region_type {
        "ibra" => Some(RegionType::Ibra),
        "imcra" => Some(RegionType::Imcra),
        _ => None,
    }
}

fn extract_region_values(values: &str) -> Vec<String> {
    values.split(",").map(|region| region.trim().to_string()).collect()
}


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = crate::database::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder().build(manager).expect("Could not build connection pool")
}
