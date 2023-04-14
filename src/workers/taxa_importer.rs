use std::path::{Path, PathBuf};

use diesel::*;
use polars::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::{index::providers::db::models::{Job, UserTaxon, UserTaxaList, Attribute, Object, ObjectValueString, AttributeParser, AttributeDataValue, AttributeDataType, ObjectValueArray, ObjectValueText}, schema};


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

    fn recv(&mut self, _cx: CX![], job: Job) {

    }

    fn term(&self, cx: CX![], panic: Option<String>) {
        if let Some(msg) = panic {
            panic!("Unexpected thread failure: {}", msg);
        }
        cx.stop();
    }

    #[instrument]
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

    import_taxa(taxa_list, &read_file(path)?, conn)?;

    Ok(())
}

pub fn read_file(file: PathBuf) -> PolarsResult<DataFrame> {
    info!(?file, "Reading");

    let schema_patch = Schema::from(
        vec![
            Field::new("year", DataType::Utf8),
            Field::new("basionymYear", DataType::Utf8),
        ].into_iter(),
    );

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

    let mut objects = Vec::new();
    let mut object_strings = Vec::new();
    let mut object_texts = Vec::new();
    let mut object_arrays = Vec::new();

    let attr_names = df.get_column_names();
    let attributes = find_attributes(&attr_names, conn)?;

    for attribute in attributes {
        let series = df.column(&attribute.name)?;
        info!(name = attribute.name, "Enumerating column");

        // if this is a common field add it to the user taxon record,
        // otherwise use the EAv table to associate the value with the taxon
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
            _ => for (idx, value) in series.iter().enumerate() {
                let value_uuid = match value.parse(&attribute) {
                    Some(AttributeDataValue::String(value)) => {
                        if value.len() > 255 {
                            panic!("value too long: {} - {}", attribute.name, value);
                        }

                        let value = ObjectValueString {
                            id: Uuid::new_v4(),
                            value,
                        };

                        let uuid = value.id;
                        object_strings.push(value);
                        Some(uuid)
                    },
                    Some(AttributeDataValue::Text(value)) => {
                        let value = ObjectValueText {
                            id: Uuid::new_v4(),
                            value,
                        };

                        let uuid = value.id;
                        object_texts.push(value);
                        Some(uuid)
                    },
                    Some(AttributeDataValue::Array(value)) => {
                        let value = ObjectValueArray {
                            id: Uuid::new_v4(),
                            value,
                        };

                        let uuid = value.id;
                        object_arrays.push(value);
                        Some(uuid)
                    },

                    _ => { None },
                };

                if let Some(uuid) = value_uuid {
                    objects.push(Object {
                        id: Uuid::new_v4(),
                        entity_id: rows[idx].id,
                        attribute_id: attribute.id,
                        value_id: uuid,
                    });

                }
            }
        }
    }

    info!("Importing common taxa fields");
    use schema::user_taxa::dsl::user_taxa;
    for chunk in rows.chunks(1000) {
        if let Err(err) = diesel::insert_into(user_taxa).values(chunk).execute(conn) {
            error!(?err, "Could not insert rows");
            panic!();
        }
    }

    info!("Importing taxa attribute strings");
    use schema::object_values_string::dsl::object_values_string;
    for chunk in object_strings.chunks(1000) {
        if let Err(err) = diesel::insert_into(object_values_string).values(chunk).execute(conn) {
            error!(?err, "Could not insert rows");
            panic!();
        }
    }
    info!("Importing taxa attribute text");
    use schema::object_values_text::dsl::object_values_text;
    for chunk in object_texts.chunks(1000) {
        if let Err(err) = diesel::insert_into(object_values_text).values(chunk).execute(conn) {
            error!(?err, "Could not insert rows");
            panic!();
        }
    }
    info!("Importing taxa attribute arrays");
    use schema::object_values_array::dsl::object_values_array;
    for chunk in object_arrays.chunks(1000) {
        if let Err(err) = diesel::insert_into(object_values_array).values(chunk).execute(conn) {
            error!(?err, "Could not insert rows");
            panic!();
        }
    }
    info!("Importing taxa attribute objects");
    use schema::objects::dsl::objects as objects_table;
    for chunk in objects.chunks(1000) {
        if let Err(err) = diesel::insert_into(objects_table).values(chunk).execute(conn) {
            error!(?err, "Could not insert rows");
            panic!();
        }
    }

    info!(rows = rows.len(), "Imported");

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
