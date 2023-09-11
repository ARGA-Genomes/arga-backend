use std::{path::PathBuf, io};
use diesel::{PgConnection, r2d2::{ConnectionManager, Pool}};

use crate::data::ncbi::name_matcher::{NameRecord, match_names};
use super::Error;


pub fn match_report(path: PathBuf) -> Result<(), Error> {
    let mut records: Vec<NameRecord> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let mut pool = Pool::builder().build(manager)?;

    let mut writer = csv::Writer::from_writer(io::stdout());
    let matched = match_names(&records, &mut pool);

    for record in records {
        let has_match = matched.contains_key(&record.scientific_name);
        writer.write_record(&[&record.scientific_name, &has_match.to_string()])?;
    }

    Ok(())
}
