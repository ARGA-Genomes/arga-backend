use std::path::{Path, PathBuf};
use std::collections::HashMap;

use diesel::*;
use polars::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::database::schema;
use crate::database::models::{Job, NameList, NameListType, Specimen};


pub struct SpecimenImporter {
    thread: PipedThread<Job, Job>,
}

impl SpecimenImporter {
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
                    let list = create_specimen_list(&data.name, &data.description);
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

pub fn create_specimen_list(list_name: &str, list_description: &Option<String>) -> NameList {
    use schema::name_lists::dsl::*;
    let conn = &mut establish_connection();

    diesel::insert_into(name_lists)
        .values((
            list_type.eq(NameListType::Specimen),
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

    import_specimens(&read_file(path.clone())?, list, conn)?;

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
struct SpecimenImport {
    scientific_name: String,
    type_status: String,
    institution_name: Option<String>,
    organism_id: Option<String>,
    locality: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    details: Option<String>,
    remarks: Option<String>,
}

#[instrument(skip(df, conn))]
fn import_specimens(df: &DataFrame, list: &NameList, conn: &mut PgConnection) -> Result<(), Error> {
    info!(height = df.height(), "Transforming");

    let mut rows = Vec::with_capacity(df.height());
    for _ in 0..df.height() {
        rows.push(SpecimenImport::default());
    }

    let series = df.column("scientificName")?;
    for (idx, value) in series.iter().enumerate() {
        rows[idx].scientific_name = parse_string(&value).expect("scientificName is mandatory");
    }

    let series = df.column("typeStatus")?;
    for (idx, value) in series.iter().enumerate() {
        rows[idx].type_status = parse_string(&value).expect("typeStatus is mandatory");
    }

    // optional fields adding more data to the conservation record
    if let Ok(series) = df.column("institutionName") {
        for (idx, value) in series.iter().enumerate() {
            rows[idx].institution_name = parse_string(&value);
        }
    }

    if let Ok(series) = df.column("locality") {
        for (idx, value) in series.iter().enumerate() {
            rows[idx].locality = parse_string(&value);
        }
    }

    // if let Ok(series) = df.column("latitude") {
    //     for (idx, value) in series.iter().enumerate() {
    //         rows[idx].latitude = parse_string(&value);
    //     }
    // }

    // if let Ok(series) = df.column("longitude") {
    //     for (idx, value) in series.iter().enumerate() {
    //         rows[idx].longitude = parse_string(&value);
    //     }
    // }

    if let Ok(series) = df.column("organismID") {
        for (idx, value) in series.iter().enumerate() {
            rows[idx].organism_id = parse_string(&value);
        }
    }

    if let Ok(series) = df.column("organismDetails") {
        for (idx, value) in series.iter().enumerate() {
            rows[idx].details = parse_string(&value);
        }
    }

    if let Ok(series) = df.column("organismRemarks") {
        for (idx, value) in series.iter().enumerate() {
            rows[idx].remarks = parse_string(&value);
        }
    }

    info!(total=rows.len(), "Importing specimens");
    use schema::{names, specimens};

    let mut total = 0;
    for chunk in rows.chunks(1000) {
        info!(rows = chunk.len(), "Inserting into specimens");

        let mut id_map: HashMap<String, Uuid> = HashMap::new();
        let all_names: Vec<&String> = rows.iter().map(|row| &row.scientific_name).collect();

        let results = names::table
            .select((names::id, names::scientific_name))
            .filter(names::scientific_name.eq_any(all_names))
            .load::<(Uuid, String)>(conn)?;

        for (uuid, name) in results {
            id_map.insert(name, uuid);
        }

        let mut values = Vec::new();
        for row in chunk {
            if let Some(uuid) = id_map.get(&row.scientific_name) {
                values.push(Specimen {
                    id: Uuid::new_v4(),
                    list_id: list.id.clone(),
                    name_id: uuid.clone(),
                    type_status: row.type_status.clone(),
                    institution_name: row.institution_name.clone(),
                    organism_id: row.organism_id.clone(),
                    locality: row.locality.clone(),
                    latitude: row.latitude,
                    longitude: row.longitude,
                    details: row.details.clone(),
                    remarks: row.remarks.clone(),
                })
            }
        }

        let inserted_rows = diesel::insert_into(specimens::table)
            .values(values)
            .execute(conn)?;

        info!(inserted_rows, "Inserted into specimens");
        total += inserted_rows;
    }

    info!(total, "Finished importing specimens");
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
