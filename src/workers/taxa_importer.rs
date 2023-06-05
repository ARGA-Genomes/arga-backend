use std::path::{Path, PathBuf};
use std::collections::HashMap;

use diesel::*;
use polars::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::database::schema;
use crate::database::models::{
    Job,
    UserTaxon,
    UserTaxaList,
    Attribute,
    AttributeParser,
    AttributeDataValue,
    AttributeDataType,
    Name,
    Regions, RegionType,
};


pub struct TaxaImporter {
    thread: PipedThread<Job, Job>,
}

impl TaxaImporter {
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
        info!("Running taxa importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let taxa_list = create_taxa_list(&data.name, &data.description);
                    let path = Path::new(&tmp_path).join(data.tmp_name);
                    import(path, &taxa_list).unwrap();
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

    #[error("an error occurred parsing the taxa file")]
    Parsing(#[from] PolarsError),
}

#[derive(Debug, Deserialize)]
struct ImportJobData {
    name: String,
    description: Option<String>,
    tmp_name: String,
}

pub fn create_taxa_list(list_name: &str, list_description: &Option<String>) -> UserTaxaList {
    use schema::user_taxa_lists::dsl::*;
    let conn = &mut establish_connection();

    diesel::insert_into(user_taxa_lists)
        .values((
            name.eq(list_name),
            description.eq(list_description),
        ))
        .get_result(conn)
        .unwrap()
}

#[instrument]
pub fn import(path: PathBuf, taxa_list: &UserTaxaList) -> Result<(), Error> {
    info!("Establishing database connection");
    let conn = &mut establish_connection();

    import_names(&read_file(path.clone())?, conn)?;
    import_taxa(taxa_list, &read_file(path.clone())?, conn)?;
    import_regions(&read_file(path.clone())?, conn)?;

    Ok(())
}

pub fn read_file(file: PathBuf) -> PolarsResult<DataFrame> {
    info!(?file, "Reading");

    let schema_patch = Schema::from_iter(vec![
        Field::new("year", DataType::Utf8),
        Field::new("basionymYear", DataType::Utf8),
    ]);

    let df = CsvReader::from_path(file)?
        .has_header(true)
        .with_delimiter(b',')
        .with_dtypes(Some(Arc::new(schema_patch)))
        .with_quote_char(Some(b'"'))
        .finish()?
        .lazy()
        .groupby(["scientificName"])
        .agg([col("*").first()])
        // .select(&[col("*")])
        .collect()?;

    Ok(df)
}


fn establish_connection() -> PgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


#[instrument(skip(conn))]
fn find_attributes(names: &Vec<&str>, conn: &mut PgConnection) -> Result<Vec<Attribute>, Error> {
    use schema::attributes::dsl::*;

    let list = attributes.filter(name.eq_any(names)).load::<Attribute>(conn)?;
    Ok(list)
}


#[instrument(skip(df, conn))]
fn import_names(df: &DataFrame, conn: &mut PgConnection) -> Result<(), Error> {
    info!(height = df.height(), "Transforming");

    let mut rows = Vec::with_capacity(df.height());
    for _ in 0..df.height() {
        rows.push(Name {
            id: Uuid::new_v4(),
            ..Default::default()
        });
    }

    // The scientific name field is mandatory for all taxa imports because we
    // maintain a unique table of names that we associate other info with. this approach
    // allows us to associate multiple sources with a unique name that gets used in the real world
    let series = df.column("scientificName")?;
    for (idx, value) in series.iter().enumerate() {
        rows[idx].scientific_name = parse_string(&value).expect("scientificName is mandatory")
    }

    // set the optional fields for the name data. it wont overwrite existing names
    // but new names will prserve these values indefinitely
    let attr_names = df.get_column_names();
    let attributes = find_attributes(&attr_names, conn)?;

    for attribute in &attributes {
        let series = df.column(&attribute.name)?;
        info!(name = attribute.name, "Enumerating column");

        match attribute.name.as_str() {
            "authority" => for (idx, value) in series.iter().enumerate() {
                rows[idx].authorship = parse_string(&value);
            },
            "canonicalName" => for (idx, value) in series.iter().enumerate() {
                rows[idx].canonical_name = parse_string(&value);
            },
            "rank" => for (idx, value) in series.iter().enumerate() {
                rows[idx].rank = parse_string(&value).unwrap_or("unranked".to_string());
            },
            _ => {}
        }
    }

    info!(total=rows.len(), "Importing taxa names");
    use schema::names::dsl::*;

    let mut total = 0;
    for chunk in rows.chunks(10_000) {
        info!(rows = chunk.len(), "Inserting into global names list");

        let inserted_rows = diesel::insert_into(names)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(conn)?;

        info!(inserted_rows, "Inserted into global names list");
        total += inserted_rows;
    }

    info!(total, "Finished importing taxa names");
    Ok(())
}


#[instrument(skip(df, conn))]
fn import_taxa(taxa_list: &UserTaxaList, df: &DataFrame, conn: &mut PgConnection) -> Result<(), Error> {
    info!(height = df.height(), "Transforming");

    let mut rows = Vec::with_capacity(df.height());
    for _ in 0..df.height() {
        rows.push(UserTaxon {
            id: Uuid::new_v4(),
            taxa_lists_id: taxa_list.id,
            ..Default::default()
        });
    }

    let attr_names = df.get_column_names();
    let attributes = find_attributes(&attr_names, conn)?;

    for attribute in attributes {
        let series = df.column(&attribute.name)?;
        info!(name = attribute.name, "Enumerating column");

        match attribute.name.as_str() {
            "scientificName" => for (idx, value) in series.iter().enumerate() {
                rows[idx].scientific_name = parse_string(&value);
            },
            "authority" => for (idx, value) in series.iter().enumerate() {
                rows[idx].scientific_name_authorship = parse_string(&value);
            },
            "canonicalName" => for (idx, value) in series.iter().enumerate() {
                rows[idx].canonical_name = parse_string(&value);
            },
            "kingdom" => for (idx, value) in series.iter().enumerate() {
                rows[idx].kingdom = parse_string(&value);
            },
            "phylum" => for (idx, value) in series.iter().enumerate() {
                rows[idx].phylum = parse_string(&value);
            },
            "class" => for (idx, value) in series.iter().enumerate() {
                rows[idx].class = parse_string(&value);
            },
            "order" => for (idx, value) in series.iter().enumerate() {
                rows[idx].order = parse_string(&value);
            },
            "family" => for (idx, value) in series.iter().enumerate() {
                rows[idx].family = parse_string(&value);
            },
            "genus" => for (idx, value) in series.iter().enumerate() {
                rows[idx].genus = parse_string(&value);
            },
            _ => {},
        }
    }

    info!(rows = rows.len(), "Importing common taxa fields");
    use schema::user_taxa::dsl::user_taxa;
    use schema::names;

    let mut total = 0;
    for chunk in rows.chunks_mut(1000) {
        let mut id_map: HashMap<String, Uuid> = HashMap::new();
        let all_names: Vec<String> = chunk.iter().map(|row| row.scientific_name.clone().unwrap()).collect();

        let results = names::table
            .select((names::id, names::scientific_name))
            .filter(names::scientific_name.eq_any(all_names))
            .load::<(Uuid, String)>(conn)?;

        for (uuid, name) in results {
            id_map.insert(name, uuid);
        }

        for row in chunk.iter_mut() {
            if let Some(name) = &row.scientific_name {
                row.name_id = id_map.get(name).expect("Cannot find name id").clone();
            }
        }

        total += diesel::insert_into(user_taxa).values(chunk.to_vec()).execute(conn)?;
    }

    info!(total, "Finished importing common taxa fields");
    Ok(())
}


impl AttributeParser for AnyValue<'_> {
    fn parse(&self, attribute: &Attribute) -> Option<AttributeDataValue> {
        match attribute.data_type {
            AttributeDataType::String => {
                match parse_string(self) {
                    Some(text) => Some(AttributeDataValue::String(text)),
                    None => None,
                }
            },
            AttributeDataType::Text => {
                match parse_string(self) {
                    Some(text) => Some(AttributeDataValue::Text(text)),
                    None => None,
                }
            },
            AttributeDataType::Integer => None,
            AttributeDataType::Boolean => None,
            AttributeDataType::Timestamp => None,
            AttributeDataType::Array => {
                match parse_array(self) {
                    Some(arr) => Some(AttributeDataValue::Array(arr)),
                    None => None,
                }
            },
        }
    }
}


#[derive(Default)]
struct RegionImport {
    scientific_name: String,
    ibra: Option<Vec<String>>,
    imcra: Option<Vec<String>>,
}

#[instrument(skip(df, conn))]
fn import_regions(df: &DataFrame, conn: &mut PgConnection) -> Result<(), Error> {
    info!(height = df.height(), "Transforming");

    let mut rows = Vec::with_capacity(df.height());
    for _ in 0..df.height() {
        rows.push(RegionImport::default());
    }

    let series = df.column("scientificName")?;
    for (idx, value) in series.iter().enumerate() {
        rows[idx].scientific_name = parse_string(&value).expect("scientificName is mandatory")
    }

    // set the optional fields for the name data. it wont overwrite existing names
    // but new names will prserve these values indefinitely
    let attr_names = df.get_column_names();
    let attributes = find_attributes(&attr_names, conn)?;

    for attribute in &attributes {
        let series = df.column(&attribute.name)?;
        info!(name = attribute.name, "Enumerating column");

        match attribute.name.as_str() {
            "ibraRegions" => for (idx, value) in series.iter().enumerate() {
                rows[idx].ibra = parse_array(&value);
            },
            "imcraRegions" => for (idx, value) in series.iter().enumerate() {
                rows[idx].imcra = parse_array(&value);
            },
            _ => {}
        }
    }

    info!(total=rows.len(), "Importing regions");
    use schema::{regions, names};

    let mut total = 0;
    for chunk in rows.chunks(10_000) {
        info!(rows = chunk.len(), "Inserting into regions");

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
                if let Some(value) = &row.ibra {
                    values.push(Regions {
                        id: Uuid::new_v4(),
                        name_id: uuid.clone(),
                        region_type: RegionType::Ibra,
                        values: value.clone(),
                    });
                }
                if let Some(value) = &row.imcra {
                    values.push(Regions {
                        id: Uuid::new_v4(),
                        name_id: uuid.clone(),
                        region_type: RegionType::Imcra,
                        values: value.clone(),
                    });
                }
            }
        }

        let inserted_rows = diesel::insert_into(regions::table)
            .values(values)
            .execute(conn)?;

        info!(inserted_rows, "Inserted into regions");
        total += inserted_rows;
    }

    info!(total, "Finished importing regions");
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

fn parse_array(value: &AnyValue) -> Option<Vec<String>> {
    match parse_string(value) {
        Some(text) => {
            let arr = text.split(",").map(|val| val.trim().to_string()).collect();
            Some(arr)
        },
        None => None,
    }
}
