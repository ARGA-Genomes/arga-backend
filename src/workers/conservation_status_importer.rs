use std::path::{Path, PathBuf};
use std::collections::HashMap;

use diesel::*;
use polars::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::database::schema;
use crate::database::models::{Job, ConservationStatus, NameList, NameListType};


pub struct ConservationStatusImporter {
    thread: PipedThread<Job, Job>,
}

impl ConservationStatusImporter {
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
        info!("Running conservation status importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let list = create_conservation_list(&data.name, &data.description);
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
    Parsing(#[from] PolarsError),
}

#[derive(Debug, Deserialize)]
struct ImportJobData {
    name: String,
    description: Option<String>,
    tmp_name: String,
}

pub fn create_conservation_list(list_name: &str, list_description: &Option<String>) -> NameList {
    use schema::name_lists::dsl::*;
    let conn = &mut establish_connection();

    diesel::insert_into(name_lists)
        .values((
            list_type.eq(NameListType::ConservationStatus),
            name.eq(list_name),
            description.eq(list_description),
        ))
        .get_result(conn)
        .unwrap()
}

#[instrument]
pub fn import(path: PathBuf, list: &NameList) -> Result<(), Error> {
    info!("Establishing database connection");
    let conn = &mut establish_connection();

    // import_names(&read_file(path.clone())?, conn)?;
    // import_taxa(taxa_list, &read_file(path.clone())?, conn)?;
    import_conservation_status(&read_file(path.clone())?, list, conn)?;

    Ok(())
}

pub fn read_file(file: PathBuf) -> PolarsResult<DataFrame> {
    info!(?file, "Reading");

    let df = CsvReader::from_path(file)?
        .has_header(true)
        .with_delimiter(b',')
        .with_quote_char(Some(b'"'))
        .finish()?
        .lazy()
        .select(&[col("*")])
        .collect()?;

    Ok(df)
}


fn establish_connection() -> PgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


#[derive(Default)]
struct ConservationStatusImport {
    canonical_name: String,
    status: String,
    state: Option<String>,
    source: Option<String>,
}

#[instrument(skip(df, conn))]
fn import_conservation_status(df: &DataFrame, list: &NameList, conn: &mut PgConnection) -> Result<(), Error> {
    info!(height = df.height(), "Transforming");

    let mut rows = Vec::with_capacity(df.height());
    for _ in 0..df.height() {
        rows.push(ConservationStatusImport::default());
    }

    let series = df.column("canonicalName")?;
    for (idx, value) in series.iter().enumerate() {
        rows[idx].canonical_name = parse_string(&value).expect("canonicalName is mandatory");
    }

    let series = df.column("conservationStatus")?;
    for (idx, value) in series.iter().enumerate() {
        rows[idx].status = parse_string(&value).expect("conservationStatus is mandatory");
    }

    // optional fields adding more data to the conservation record
    if let Ok(series) = df.column("state") {
        for (idx, value) in series.iter().enumerate() {
            rows[idx].state = parse_string(&value);
        }
    }

    if let Ok(series) = df.column("source") {
        for (idx, value) in series.iter().enumerate() {
            rows[idx].source = parse_string(&value);
        }
    }

    info!(total=rows.len(), "Importing conservation statuses");
    use schema::{names, conservation_statuses};

    let mut total = 0;
    for chunk in rows.chunks(10_000) {
        info!(rows = chunk.len(), "Inserting into conservation_statuses");

        let mut id_map: HashMap<String, Uuid> = HashMap::new();
        let all_names: Vec<&String> = rows.iter().map(|row| &row.canonical_name).collect();

        let results = names::table
            .select((names::id, names::canonical_name))
            .filter(names::canonical_name.eq_any(all_names))
            .load::<(Uuid, Option<String>)>(conn)?;

        for (uuid, name) in results {
            if let Some(name) = name {
                id_map.insert(name, uuid);
            }
        }

        let mut values = Vec::new();
        for row in chunk {
            if let Some(uuid) = id_map.get(&row.canonical_name) {
                values.push(ConservationStatus {
                    id: Uuid::new_v4(),
                    list_id: list.id.clone(),
                    name_id: uuid.clone(),
                    status: row.status.clone(),
                    state: row.state.clone(),
                    source: row.source.clone(),
                })
            }
        }

        let inserted_rows = diesel::insert_into(conservation_statuses::table)
            .values(values)
            .execute(conn)?;

        info!(inserted_rows, "Inserted into conservation_statuses");
        total += inserted_rows;
    }

    info!(total, "Finished importing conservation statuses");
    Ok(())
}


fn parse_string(value: &AnyValue) -> Option<String> {
    match value {
        AnyValue::Utf8(text) => {
            if text.trim().is_empty() {
                None
            } else {
                Some(String::from(text.to_string()))
            }
        },
        _ => None,
    }
}
