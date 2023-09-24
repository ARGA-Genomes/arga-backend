use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    assembly_date: Option<String>,
    assembly_method: Option<String>,
    assembly_method_version: Option<String>,
    assembly_method_version_or_date: Option<String>,
    assembly_name: Option<String>,
    genome_size: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssemblyEvent {
    record_id: String,
    event_date: Option<String>,
    assembly_type: Option<String>,
    version_status: Option<String>,
    name: Option<String>,
    genome_size: Option<String>,
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("assemblies.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let version_status = record.assembly_method_version.or(record.assembly_method_version_or_date);

        let event = AssemblyEvent {
            record_id: record.id,
            event_date: record.assembly_date,
            assembly_type: record.assembly_method,
            version_status,
            name: record.assembly_name,
            genome_size: record.genome_size,
        };

        writer.serialize(event)?;
    }

    Ok(())
}
