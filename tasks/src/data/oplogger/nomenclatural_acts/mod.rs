use std::path::PathBuf;

use arga_core::models::{Operation, Atom};
use arga_core::schema;
use chrono::Utc;
use diesel::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::Deserialize;
use xxhash_rust::xxh3::Xxh3;

use crate::data::{Error, oplogger::ObjectFrame};

use super::{hlc::HybridTimestamp, Version};


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub acted_on: String,
    pub scientific_name: String,
    pub source_url: String,
}


pub struct OriginalDescription {
    pub acted_on: String,
    pub scientific_name: String,
    pub source_url: String,
}

impl From<Record> for OriginalDescription {
    fn from(value: Record) -> Self {
        Self {
            acted_on: value.acted_on,
            scientific_name: value.scientific_name,
            source_url: value.source_url,
        }
    }
}


pub struct NomenclaturalActs {
    pub path: PathBuf,
}

impl NomenclaturalActs {
    pub fn original_descriptions(&self) -> Result<Vec<Operation>, Error> {
        let mut records: Vec<Record> = Vec::new();
        for row in csv::Reader::from_path(&self.path)?.deserialize() {
            records.push(row?);
        }

        let ts: HybridTimestamp = Utc::now().into();
        let mut version = Version(ts);
        let mut operations = Vec::new();

        for record in records.into_iter() {
            // the uniqueness of a nomenclatural act is the acted on taxon,
            // the scientific name, and the act itself.
            let mut hasher = Xxh3::new();
            hasher.update(record.acted_on.as_bytes());
            hasher.update(record.scientific_name.as_bytes());
            let hash = hasher.digest();

            let mut object = ObjectFrame::new(version, hash.to_string());

            if record.acted_on == record.scientific_name {
                object.update(Atom::ActedOn { value: "Biota".to_string() });
            } else {
                object.update(Atom::ActedOn { value: record.acted_on });
            }

            object.update(Atom::ScientificName { value: record.scientific_name });
            object.update(Atom::SourceUrl { value: record.source_url });

            operations.extend(object.operations);
            version = object.previous;
        }

        Ok(operations)
    }
}


pub fn process(path: PathBuf) -> Result<(), Error> {
    let acts = NomenclaturalActs { path };
    let records = acts.original_descriptions()?;
    import_operations(records)?;

    Ok(())
}


fn import_operations(records: Vec<Operation>) -> Result<(), Error> {
    use schema::operation_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    for chunk in records.chunks(1000) {
        diesel::insert_into(operation_logs)
            .values(chunk)
            .execute(&mut conn)?;
    }

    Ok(())
}


fn get_pool() -> Result<PgPool, Error> {
    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}
