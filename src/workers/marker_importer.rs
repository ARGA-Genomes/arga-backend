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
use crate::database::models::{Job, NameList, NameListType, Marker};

type PgPool = Pool<ConnectionManager<PgConnection>>;


pub struct MarkerImporter {
    thread: PipedThread<Job, Job>,
}

impl MarkerImporter {
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
        info!("Running specimen importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let list = create_marker_list(&data.name, &data.description).unwrap();
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
struct Record {
    accession: String,
    #[serde(rename(deserialize = "scientificName"))]
    scientific_name: String,
    // details: Option<String>,
    version: Option<String>,
    basepairs: Option<usize>,
    r#type: Option<String>,
    shape: Option<String>,
    // date: Option<String>,
    // comment: Option<String>,
    genbank_url: Option<String>,
    fasta_url: Option<String>,

    // arbitrary attributes that may or may not be
    // appropriate to be extracted into a common field
    extra_data: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Deserialize)]
struct NameMatch {
    id: Uuid,
    scientific_name: String,
    canonical_name: Option<String>,
}


pub fn create_marker_list(list_name: &str, list_description: &Option<String>) -> Result<NameList, Error> {
    use schema::name_lists::dsl::*;
    let pool = get_connection_pool();
    let mut conn = pool.get()?;

    let name_list = diesel::insert_into(name_lists)
        .values((
            list_type.eq(NameListType::Marker),
            name.eq(list_name),
            description.eq(list_description),
        ))
        .get_result(&mut conn)?;

    Ok(name_list)
}

#[instrument]
pub fn import(path: PathBuf, list: &NameList) -> Result<(), Error> {
    info!("Getting database connection pool");
    let pool = &mut get_connection_pool();

    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    import_markers(records, list, pool)?;

    Ok(())
}


fn import_markers(records: Vec<Record>, list: &NameList, pool: &mut PgPool) -> Result<(), Error> {
    use schema::markers;

    let names = match_names(&records, pool);
    let markers = extract_markers(&list, &names, &records);

    // filter out unmatched markers
    let markers = markers.into_iter().filter_map(|r| r).collect::<Vec<Marker>>();

    info!(total=markers.len(), "Importing markers");
    let imported: Vec<Result<usize, Error>> = markers.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(markers::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=markers.len(), total_imported, "Importing markers finished");

    Ok(())
}


fn match_names(records: &Vec<Record>, pool: &mut PgPool) -> HashMap<String, Uuid> {
    use schema::names;
    info!(total=records.len(), "Matching names");

    let matched: Vec<Result<Vec<NameMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let names: Vec<&String> = chunk.iter().map(|row| &row.scientific_name).collect();

        // we would prefer to match on scientific name but sometimes the source
        // records only provide the canonical name so try to match on either scientific
        // or canonical name and do double entry in the map to allow for lookups from either
        let results = names::table
            .select((names::id, names::scientific_name, names::canonical_name))
            .filter(names::scientific_name.eq_any(&names))
            .or_filter(names::canonical_name.eq_any(&names))
            .load::<NameMatch>(&mut conn)?;

        Ok::<Vec<NameMatch>, Error>(results)
    }).collect();

    let mut id_map: HashMap<String, Uuid> = HashMap::new();

    for chunk in matched {
        if let Ok(names) = chunk {
            for name_match in names {
                id_map.insert(name_match.scientific_name, name_match.id);
                if let Some(canonical_name) = name_match.canonical_name {
                    id_map.insert(canonical_name, name_match.id);
                }
            }
        }
    }

    info!(total=records.len(), matched=id_map.len(), "Matching names finished");
    id_map
}


fn extract_markers(list: &NameList, names: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<Marker>> {
    info!(total=records.len(), "Extracting markers");

    let markers = records.par_iter().map(|row| {
        match names.get(&row.scientific_name) {
            Some(name_id) => Some(Marker {
                id: Uuid::new_v4(),
                list_id: list.id.clone(),
                name_id: name_id.clone(),
                created_at: chrono::DateTime::default(),
                updated_at: chrono::DateTime::default(),

                accession: row.accession.clone(),
                material_sample_id: None,
                gb_acs: None,
                marker_code: None,
                nucleotide: None,
                recorded_by: None,
                version: row.version.clone(),
                basepairs: row.basepairs.map(|v| v as i64),
                type_: row.r#type.clone(),
                shape: row.shape.clone(),
                source_url: row.genbank_url.clone(),
                fasta_url: row.fasta_url.clone(),
                extra_data: row.extra_data.clone(),
            }),
            None => None,
        }
    }).collect::<Vec<Option<Marker>>>();

    info!(markers=markers.len(), "Extracting marker finished");
    markers
}


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = crate::database::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder().build(manager).expect("Could not build connection pool")
}
